use clap::Parser;
use execution_time::ExecutionTime;
use rayon::prelude::*;

use std::{
    collections::HashSet,
    io::BufRead,
    path::PathBuf,
    process,
    sync::atomic::{AtomicUsize, Ordering},
};

// Functions defined in lib.rs
use unique::*;

const CHUNK_SIZE: usize = 10_000;
const NEWLINE_BYTE: u8 = b'\n';

/*
Inspiração: uniq, huniq e semiuniq.
https://github.com/koraa/huniq/blob/main/src/main.rs
https://github.com/kljensen/semiuniq

clear && cargo test -- --nocapture
clear && cargo run -- -h
cargo b -r && cargo install --path=.

Enum NumberFormat: O usuário tem controle total via CLI:
unique arquivo.csv --csv -f (Padrão: 1.234,56 -> 1234.56)
unique arquivo.csv --csv -f --number-format international (Padrão: 1,234.56 -> 1234.56)
*/

fn main() -> UniqueResult<()> {
    if let Err(error) = run() {
        eprintln!("Operation failed!\n{}", error);
        process::exit(1); // Explicitly exit with failure code
    }
    Ok(())
}

fn run() -> UniqueResult<()> {
    let timer = ExecutionTime::start();
    let arguments: Arguments = Arguments::parse();

    // The input is a file or standard input (stdin)
    let input_file: Option<PathBuf> = arguments.file.clone();
    let mut buffer: Box<dyn BufRead> = read_file_or_stdin(&input_file)?;

    let mut delimiter_set: HashSet<usize> = HashSet::new();
    let mut uniq_hashes: HashSet<String> = HashSet::new();
    let mut num_repeated_lines: usize = 0;

    // Shared atomic counter for empty lines across parallel threads
    let atomic_empty_lines = AtomicUsize::new(0);

    let mut line_number: usize = 0;
    let mut num_bytes: usize = 1;

    // Stores the CSV header used for Serde mapping (DocsFiscais)
    let mut header_string = String::new();

    // --- STEP 1: HEADER TREATMENT (Finding the first non-empty line) ---
    if arguments.parse_csv_file {
        loop {
            let mut header_bytes: Vec<u8> = Vec::new();
            num_bytes = buffer.read_until(NEWLINE_BYTE, &mut header_bytes)?;

            // Exit if end of file (EOF) is reached without finding content
            if num_bytes == 0 {
                break;
            }

            let line_utf8 = get_string_utf8_from_slice_bytes(&header_bytes)?;

            // Skip and COUNT empty lines at the very beginning
            if line_utf8.trim().is_empty() {
                // We count them so the final report remains accurate
                atomic_empty_lines.fetch_add(1, Ordering::Relaxed);
                continue;
            }

            // HEADER FOUND: The first non-empty line
            line_number += 1;
            header_string = line_utf8;

            // Print the header immediately unless we are in "repeated lines only" mode
            if !arguments.only_print_repeated_lines {
                println!("{}", header_string);
            }

            // Track column count for later validation
            let header_cols = header_string.split(arguments.separator).count();
            delimiter_set.insert(header_cols);

            break; // Header logic finished, proceed to data
        }
    }

    // --- STEP 2: DATA PROCESSING (Chunked Parallel Loop) ---
    while num_bytes > 0 {
        let mut vec_lines: Vec<(usize, Vec<u8>)> = Vec::new();

        // Fill a chunk of lines to process in parallel
        loop {
            let mut line: Vec<u8> = Vec::new();
            num_bytes = buffer.read_until(NEWLINE_BYTE, &mut line)?;
            if num_bytes == 0 {
                // EOF is reached
                break;
            }
            line_number += 1;
            vec_lines.push((line_number, line));
            if vec_lines.len() >= CHUNK_SIZE {
                break;
            }
        }

        // PARALLEL ENGINE: Process the chunk using Rayon
        let vector: AnalysisResult = vec_lines
            .into_par_iter() // rayon: parallel iterator
            .map(|(line_number, vec_bytes)| {
                let line_utf8: String = get_string_utf8_from_slice_bytes(&vec_bytes)?;
                let is_empty = line_utf8.trim().is_empty();

                // 1. Handle Empty Lines
                if is_empty {
                    atomic_empty_lines.fetch_add(1, Ordering::Relaxed);
                    if arguments.remove_empty_lines {
                        return Ok(None);
                    }
                    return Ok(Some(AnalyzedLine::empty(line_number)));
                }

                // 2. Handle Data Lines
                let (new_line, num_cols) = if arguments.map_docs_fiscais {
                    analise_line_with_serde(&arguments, &line_utf8, &header_string)?
                } else {
                    analise_line(&arguments, &line_utf8)?
                };

                Ok(Some(AnalyzedLine {
                    line_number,
                    content: new_line,
                    column_count: num_cols,
                    is_empty: false,
                }))
            })
            .collect();

        // Não é necessário ordenar, pois rayon coleta já ordenado!
        // No sorting required as rayon collects already sorted!
        // vector.par_sort_by_key(|&(line_number,..)| line_number);

        // --- STEP 3: APPLY RESULTS ---
        for analyzed in vector?.into_iter().flatten() {
            apply_analysis(
                &arguments,
                &analyzed.content,
                analyzed.column_count,
                &mut num_repeated_lines,
                &mut uniq_hashes,
                &mut delimiter_set,
            );
        }
    }

    // Sync the total empty lines count from the atomic counter
    let num_empty_lines = atomic_empty_lines.load(Ordering::Relaxed);

    // Final CSV structure analysis
    analise_csv_file(&arguments, delimiter_set);

    // Show statistics (Unique, Repeated, Total, Run Time)
    print_verbose(
        &arguments,
        timer,
        uniq_hashes,
        num_repeated_lines,
        num_empty_lines,
    );

    Ok(())
}

fn apply_analysis(
    args: &Arguments,
    line: &str,
    num_cols: usize,
    num_repeated_lines: &mut usize,
    uniq_hashes: &mut HashSet<String>,
    delimiter_set: &mut HashSet<usize>,
) {
    let mut filter: String = line.to_owned();

    if args.ignore_case {
        filter = filter.to_lowercase();
    }

    let hash: String = blake3::hash(filter.as_bytes()).to_string();

    if uniq_hashes.insert(hash) {
        if args.remove_empty_lines && line.trim().is_empty() {
            return;
        }
        if !args.only_print_repeated_lines {
            println!("{line}");
        }
    } else {
        if args.only_print_repeated_lines {
            println!("{line}");
        }
        *num_repeated_lines += 1;
    }

    if args.parse_csv_file {
        delimiter_set.insert(num_cols);
    }
}

fn analise_csv_file(args: &Arguments, delimiter_set: HashSet<usize>) {
    let separator: char = args.separator;

    if args.parse_csv_file {
        // Coleta do HashSet para Vector
        let mut delimiter_vec = delimiter_set.into_iter().collect::<Vec<usize>>();
        delimiter_vec.sort();

        // Verifica se há mais de um número de colunas (inconsistência) ou zero colunas
        if delimiter_vec.len() != 1 || delimiter_vec.contains(&0) {
            eprintln!();
            eprintln!("❌ Invalid CSV file!");
            eprintln!("   • CSV column separator: '{}'", separator);
            eprintln!("   • Column counts observed in rows: {:?}", delimiter_vec);
        } else if args.verbose {
            let first_element = delimiter_vec[0];
            eprintln!();
            eprintln!("✅ Valid CSV file!");
            eprintln!("   • CSV column separator: '{}'", separator);
            eprintln!("   • Constant column count: {}", first_element);
        }
    }
}

fn print_verbose(
    args: &Arguments,
    timer: ExecutionTime,
    uniq_hashes: HashSet<String>,
    num_repeated_lines: usize,
    num_empty_lines: usize,
) {
    let elapsed = timer.get_duration();
    let num_unique_lines = uniq_hashes.len();
    let num_total_lines_original = num_unique_lines + num_repeated_lines;

    // Cálculo da taxa de redução (deduplicação)
    let reduction_percent = if num_total_lines_original > 0 {
        (num_repeated_lines as f64 / num_total_lines_original as f64) * 100.0
    } else {
        0.0
    };

    // Cálculo de vazão (throughput)
    let lines_per_sec = if elapsed.as_secs_f64() > 0.0 {
        (num_total_lines_original as f64 / elapsed.as_secs_f64()) as usize
    } else {
        0
    };

    // Linhas finais no arquivo de saída
    let num_total_lines_final = if args.remove_empty_lines && num_empty_lines > 0 {
        num_unique_lines.saturating_sub(1)
    } else {
        num_unique_lines
    };

    let max_len = num_total_lines_original.to_string().len().max(10);

    if args.verbose {
        eprintln!("\n{}", "=".repeat(45));
        eprintln!("📊 EXECUTIONS SUMMARY");
        eprintln!("{}", "=".repeat(45));

        eprintln!("📝 INPUT STATS:");
        eprintln!(
            "   • Total lines processed : {:>max_len$}",
            num_total_lines_original
        );
        eprintln!("   • Total empty lines     : {:>max_len$}", num_empty_lines);

        eprintln!("\n🔍 PROCESSING DETAILS:");
        eprintln!(
            "   • Unique lines found    : {:>max_len$}",
            num_unique_lines
        );
        eprintln!(
            "   • Repeated lines removed: {:>max_len$}",
            num_repeated_lines
        );
        eprintln!(
            "   • Deduplication rate    : {:>max_len$.2}%",
            reduction_percent
        );

        eprintln!("\n💾 OUTPUT STATS:");
        eprintln!(
            "   • Lines in final file   : {:>max_len$}",
            num_total_lines_final
        );

        eprintln!("\n⏱️  PERFORMANCE:");
        eprintln!("   • Total run time        : {:?}", elapsed);
        eprintln!("   • Throughput            : {} lines/sec", lines_per_sec);
        eprintln!("{}\n", "=".repeat(45));
    }
}
