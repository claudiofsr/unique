use clap::Parser;
use csv::StringRecord;
use execution_time::ExecutionTime;
use rayon::prelude::*;

use std::{
    collections::HashSet,
    io::BufRead,
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

/// Main orchestration logic for the unique line processor.
///
/// This function coordinates the reading, parallel processing,
/// and sequential deduplication of lines.
fn run() -> UniqueResult<()> {
    let timer = execution_time::ExecutionTime::start();
    let arguments: Arguments = Arguments::parse();

    // Initialize input buffer (File or Stdin)
    let mut buffer: Box<dyn BufRead> = read_file_or_stdin(&arguments.file)?;

    // Shared state for tracking uniqueness and consistency
    let mut delimiter_set: HashSet<usize> = HashSet::new();
    let mut uniq_hashes: HashSet<String> = HashSet::new();
    let mut num_repeated_lines: usize = 0;
    let atomic_empty_lines = AtomicUsize::new(0);

    // CSV Header management
    let mut header_record: Option<StringRecord> = None;
    let mut line_number: usize = 0;
    let mut num_bytes: usize = 1; // Control variable for the read loops

    // --- STEP 1: HEADER TREATMENT ---
    if arguments.parse_csv_file {
        loop {
            let mut header_bytes: Vec<u8> = Vec::new();
            num_bytes = buffer.read_until(NEWLINE_BYTE, &mut header_bytes)?;

            if num_bytes == 0 {
                break;
            } // EOF reached before finding a header

            let header_string = get_string_utf8_from_slice_bytes(&header_bytes)?;

            // Skip and count empty lines before the header
            if header_string.trim().is_empty() {
                atomic_empty_lines.fetch_add(1, Ordering::Relaxed);
                continue;
            }

            // Header Found: Parse into StringRecord for Serde context
            line_number += 1;

            let h_record = StringRecord::from_iter(header_string.split(arguments.separator));

            if !arguments.only_print_repeated_lines {
                println!("{}", header_string);
            }

            delimiter_set.insert(h_record.len());
            header_record = Some(h_record); // Persist header context for the processing loop
            break;
        }
    }

    // --- STEP 2: CHUNKED PARALLEL PROCESSING ---
    while num_bytes > 0 {
        let mut vec_lines: Vec<(usize, Vec<u8>)> = Vec::with_capacity(CHUNK_SIZE);

        // Fill chunk sequentially from the buffer
        for _ in 0..CHUNK_SIZE {
            let mut line: Vec<u8> = Vec::new();
            num_bytes = buffer.read_until(NEWLINE_BYTE, &mut line)?;
            if num_bytes == 0 {
                // EOF is reached
                break;
            }
            line_number += 1;
            vec_lines.push((line_number, line));
        }

        if vec_lines.is_empty() {
            break;
        }

        // Process chunk in parallel: transformation + hashing
        let processed_chunk: UniqueResult<Vec<Option<(AnalyzedLine, String)>>> = vec_lines
            .into_par_iter() // rayon: parallel iterator
            .map(|(line_number, bytes)| {
                let line_utf8 = get_string_utf8_from_slice_bytes(&bytes)?;

                // 1. Handle Empty Lines
                if line_utf8.trim().is_empty() {
                    atomic_empty_lines.fetch_add(1, Ordering::Relaxed);
                    return Ok(if arguments.remove_empty_lines {
                        None
                    } else {
                        Some((AnalyzedLine::empty(line_number), String::new()))
                    });
                }

                // 2. Handle Data Lines
                // Choose the appropriate processing engine
                let (content, num_cols) = if arguments.map_docs_fiscais {
                    analise_line_with_serde(&line_utf8, &header_record, &arguments)?
                } else {
                    analise_line(&line_utf8, &arguments)?
                };

                // Generate hash for deduplication
                let mut filter = content.clone();
                if arguments.ignore_case {
                    filter = filter.to_lowercase();
                }
                let hash = blake3::hash(filter.as_bytes()).to_string();

                Ok(Some((
                    AnalyzedLine {
                        line_number,
                        content,
                        column_count: num_cols,
                        is_empty: false,
                    },
                    hash,
                )))
            })
            .collect();

        // --- STEP 3: SEQUENTIAL OUTPUT AND DEDUPLICATION ---
        for item in processed_chunk?.into_iter().flatten() {
            let (analyzed, hash) = item;

            if uniq_hashes.insert(hash) {
                // New unique line found
                if !arguments.only_print_repeated_lines {
                    println!("{}", analyzed.content);
                }
            } else {
                // Duplicate line found
                if arguments.only_print_repeated_lines {
                    println!("{}", analyzed.content);
                }
                num_repeated_lines += 1;
            }

            if arguments.parse_csv_file {
                delimiter_set.insert(analyzed.column_count);
            }
        }
    }

    // --- STEP 4: FINAL REPORT ---

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
        eprintln!("   • Total run time        : {}", timer.get_elapsed_time());
        eprintln!("   • Throughput            : {} lines/sec", lines_per_sec);
        eprintln!("{}\n", "=".repeat(45));
    }
}
