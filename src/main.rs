use args::Arguments;
use clap::Parser;
use claudiofsr_lib::StrExtension;
use csv::{ReaderBuilder, WriterBuilder};
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use execution_time::ExecutionTime;
use rayon::prelude::*;

use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
    process,
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

/// Represents the result of a single line analysis
#[derive(Debug)]
pub struct AnalyzedLine {
    pub line_number: usize,
    pub content: String,
    pub column_count: usize,
    pub is_empty: bool,
}

// Alias opcional para simplificar a assinatura da função
pub type AnalysisResult = UniqueResult<Vec<Option<AnalyzedLine>>>;

fn main() -> UniqueResult<()> {
    if let Err(error) = run() {
        eprintln!("Operation failed!\n{}", error);
        process::exit(1); // Explicitly exit with failure code
    }
    Ok(())
}

use std::sync::atomic::{AtomicUsize, Ordering};

fn run() -> UniqueResult<()> {
    let timer = ExecutionTime::start();
    let arguments: Arguments = Arguments::parse();

    // The input is a file or standard input (stdin)
    let input_file: Option<PathBuf> = arguments.file.clone();
    let mut buffer: Box<dyn BufRead> = read_file_or_stdin(&input_file);

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

            let line_utf8 = get_string_utf8_from_slice_bytes(&header_bytes);

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
                let line_utf8: String = get_string_utf8_from_slice_bytes(&vec_bytes);
                let is_empty = line_utf8.trim().is_empty();

                // 1. Handle Empty Lines
                if is_empty {
                    atomic_empty_lines.fetch_add(1, Ordering::Relaxed);
                    if arguments.remove_empty_lines {
                        return Ok(None);
                    }
                    return Ok(Some(AnalyzedLine {
                        line_number,
                        content: String::new(),
                        column_count: 0,
                        is_empty: true,
                    }));
                }

                // 2. Handle Data Lines
                let (new_line, num_cols) = if arguments.map_docs_fiscais {
                    analise_line_with_serde(&arguments, &line_utf8, &header_string)?
                } else {
                    analise_line(&arguments, &line_utf8, line_number)?
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

fn read_file_or_stdin(path: &Option<PathBuf>) -> Box<dyn BufRead> {
    // If we don't receive an input file, use stdin.
    let buffer: Box<dyn BufRead> = match path {
        Some(filename) => {
            let file: File = match fs::OpenOptions::new()
                .read(true)
                .write(false)
                .create(false)
                .open(filename)
            {
                Ok(file) => file,
                Err(error) => {
                    panic!("Failed to open file {filename:?}\n{error}");
                }
            };
            Box::new(BufReader::new(file))
        }
        None => Box::new(BufReader::new(io::stdin())),
    };

    buffer
}

fn get_string_utf8_from_slice_bytes(slice_bytes: &[u8]) -> String {
    let mut vec_bytes: Vec<u8> = slice_bytes.to_vec();
    // remove new line: "\r\n" or "\n"
    vec_bytes.retain(|&byte| byte != b'\r' && byte != b'\n');

    // from_utf8() checks to ensure that the bytes are valid UTF-8
    let string_utf8: String = match std::str::from_utf8(&vec_bytes) {
        Ok(str) => str.to_string(),
        Err(error1) => {
            let mut data = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(vec_bytes.as_slice());

            let mut buffer = String::new();
            let _number_of_bytes = match data.read_to_string(&mut buffer) {
                Ok(num) => num,
                Err(error2) => {
                    eprintln!("Problem reading data from file in buffer!");
                    eprintln!("Used encoding type: WINDOWS_1252.");
                    eprintln!("Try another encoding type!");
                    panic!("Failed to convert data from WINDOWS_1252 to UTF-8!: {error1}\n{error2}")
                }
            };

            buffer
        }
    };

    // remove new line: "\r\n" or "\n"
    // string_utf8.trim_end_matches(&['\r','\n']).to_string()
    string_utf8
}

/// Standard line analysis.
///
/// This function performs a generic CSV parsing. It splits the line into columns
/// based on the provided separator, applies optional formatting (date, key, numbers),
/// and rebuilds the line using a semicolon (`;`) as the delimiter.
/// Finally, it applies whitespace normalization and trimming if requested.
pub fn analise_line(
    args: &Arguments,
    line: &str,
    _line_number: usize,
) -> UniqueResult<(String, usize)> {
    let mut modified_line: String = line.to_owned();
    let mut num_cols: usize = 0;

    if args.parse_csv_file {
        let cols: Vec<String> = parse_csv_line(&modified_line, args);

        // Reconstruct the CSV line with standard settings
        // Using semicolon ';' for consistency in output/hashing
        let mut wtr = WriterBuilder::new()
            .delimiter(b';')
            .has_headers(false)
            .flexible(false)
            .from_writer(vec![]);

        wtr.write_record(&cols)?;

        modified_line = String::from_utf8(wtr.into_inner().map_err(|e| e.into_error())?)?;
        num_cols = cols.len();
    }

    // Common post-processing block
    if args.replace_multiple_whitespaces {
        modified_line = modified_line.replace_multiple_whitespaces();
    }

    if args.trim_line {
        modified_line = modified_line.trim().to_string();
    }

    Ok((modified_line, num_cols))
}

/// Helper function for parse_csv_line.
///
/// Parses a single CSV line into a Vector of Strings, applying column-level
/// replacements and formatting.
fn parse_csv_line(line: &str, args: &Arguments) -> Vec<String> {
    let mut reader = ReaderBuilder::new()
        .quoting(true)
        .double_quote(true)
        .has_headers(false)
        .trim(csv::Trim::All)
        .flexible(false)
        .delimiter(args.separator as u8)
        .from_reader(line.as_bytes());

    reader
        .records()
        .next()
        .and_then(|res| res.ok())
        .map(|record| {
            record
                .iter()
                .map(|col| {
                    // Normalize internal line breaks often found in messy CSVs
                    let mut s = col.replace("\\n", " ");
                    if args.format_date {
                        s = format_date(s);
                    }
                    if args.format_key {
                        s = format_key(s);
                    }
                    if args.format_number {
                        s = format_number(s, args.number_format);
                    }
                    s
                })
                .collect()
        })
        .unwrap_or_default()
}

// https://github.com/BurntSushi/rust-csv
#[allow(dead_code)]
fn parse_csv_with_serde(
    line: &str,
    args: &Arguments,
    line_number: usize,
) -> UniqueResult<Vec<DocsFiscais>> {
    let mut reader = ReaderBuilder::new()
        .quoting(true)
        .double_quote(true)
        .has_headers(false)
        .trim(csv::Trim::All)
        .flexible(false)
        .delimiter(args.separator as u8)
        .from_reader(line.as_bytes());

    //println!("reader {reader:?}\n");

    let mut records: Vec<DocsFiscais> = Vec::new();

    for result in reader.deserialize() {
        if line_number == 1 {
            continue;
        }
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: DocsFiscais = result?;
        //println!("{:?}", record);
        records.push(record);
    }

    Ok(records)
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
        // Collecting from HashSet to Vector
        let mut delimiter_vec = delimiter_set.into_iter().collect::<Vec<usize>>();
        delimiter_vec.sort();

        if delimiter_vec.len() != 1 || delimiter_vec.contains(&0) {
            eprintln!();
            eprintln!("Invalid CSV file!");
            eprintln!("CSV column delimiter: '{separator}'");
            eprintln!("Column delimiter number observed in rows: {delimiter_vec:?}");
        } else if args.verbose {
            let first_element = delimiter_vec[0];
            eprintln!();
            eprintln!("Valid CSV file!");
            eprintln!("CSV column delimiter: '{separator}'");
            eprintln!("Column delimiter number observed in rows: {first_element}");
        }
    }
}

/// Robust line analysis using the `DocsFiscais` struct mapping.
///
/// This function creates a virtual two-line CSV (header + current line) to allow
/// `serde` to map columns by name rather than position. This ensures high
/// robustness against schema changes.
///
/// It leverages the struct's internal `deserialize_with` logic for data
/// validation and then serializes the record back to a semicolon-delimited string.
pub fn analise_line_with_serde(
    args: &Arguments,
    line: &str,
    header: &str,
) -> UniqueResult<(String, usize)> {
    // 1. Prepare a virtual mini-CSV to provide context to the Serde deserializer
    let virtual_csv = format!("{}\n{}", header, line);

    let mut reader = ReaderBuilder::new()
        .quoting(true)
        .double_quote(true)
        .has_headers(true)
        .trim(csv::Trim::All)
        .delimiter(args.separator as u8)
        .from_reader(virtual_csv.as_bytes());

    let headers = reader.headers()?.clone();
    let num_cols = headers.len();
    let mut modified_line = String::new();

    // 2. Map the CSV data to the robust DocsFiscais struct
    // The loop iterates only once for the single data line provided
    for result in reader.deserialize::<DocsFiscais>() {
        match result {
            Ok(record) => {
                // Writer configuration identical to standard analise_line
                let mut wtr = WriterBuilder::new()
                    .delimiter(b';')
                    .has_headers(false)
                    .flexible(false)
                    .from_writer(vec![]);

                wtr.serialize(record)?;

                let bytes = wtr.into_inner().map_err(|e| e.into_error())?;
                // Remove trailing newline added by the CSV writer
                modified_line = String::from_utf8(bytes)?.trim_end().to_string();
            }
            Err(e) => {
                let err_msg = e.to_string();

                // Attempt to identify the failing column by extracting the index from the error message
                let col_info = if let Some(idx) = extract_field_index(&err_msg) {
                    headers
                        .get(idx)
                        .map(|name| format!("column '{}' (index {})", name, idx))
                        .unwrap_or_else(|| format!("index {}", idx))
                } else {
                    "field with incompatible format".to_string()
                };

                return Err(UniqueError::Mapping(format!(
                    "Failed at {}; Error: {}",
                    col_info, err_msg
                )));
            }
        }
    }

    // 3. Post-processing (Identical to standard analise_line)
    // Ensures hashing consistency regardless of the analysis engine used
    if args.replace_multiple_whitespaces {
        modified_line = modified_line.replace_multiple_whitespaces();
    }

    if args.trim_line {
        modified_line = modified_line.trim().to_string();
    }

    Ok((modified_line, num_cols))
}

/// Helper mais robusto para capturar o índice da coluna
fn extract_field_index(msg: &str) -> Option<usize> {
    // Procura por "field " seguido de dígitos
    let keyword = "field ";
    if let Some(start) = msg.find(keyword) {
        let after_field = &msg[start + keyword.len()..];
        let end = after_field
            .find(|c: char| !c.is_numeric())
            .unwrap_or(after_field.len());
        let num_str = &after_field[..end];
        return num_str.parse::<usize>().ok();
    }
    None
}

fn print_verbose(
    args: &Arguments,
    timer: ExecutionTime,
    uniq_hashes: HashSet<String>,
    num_repeated_lines: usize,
    num_empty_lines: usize,
) {
    // cat file | wc -l
    // 1. O número de hashes únicos no Set inclui o hash de uma linha vazia (se houver alguma no arquivo)
    let num_unique_lines = uniq_hashes.len();

    // 2. O total original é a soma de:
    //    Unique (non-empty) + Repeated (non-empty) + Total Empty
    //    Nota: Na lógica de hashing, se houverem 10 linhas vazias,
    //    1 entra em uniq_hashes e 9 entram em num_repeated_lines.
    //    Para simplificar o relatório, tratamos num_empty_lines como um bloco separado.
    let num_total_lines_original = num_unique_lines + num_repeated_lines;

    // 3. Cálculo da largura máxima para alinhamento visual
    let max_len = num_total_lines_original.to_string().len();

    // 4. Cálculo das linhas no arquivo final:
    //    Se remove_empty_lines for TRUE: tiramos a entrada da linha vazia do total de únicos.
    //    Se for FALSE: o num_unique_lines já representa o arquivo final (deduplicado).
    let num_total_lines_final = if args.remove_empty_lines && num_empty_lines > 0 {
        num_unique_lines.saturating_sub(1)
    } else {
        num_unique_lines
    };

    if args.verbose {
        eprintln!();
        eprintln!("Number of lines in the original file: {num_total_lines_original:>max_len$}");
        eprintln!("Number of unique lines              : {num_unique_lines:>max_len$}");
        eprintln!("Number of repeated lines            : {num_repeated_lines:>max_len$}");
        eprintln!("Number of empty lines               : {num_empty_lines:>max_len$}");
        eprintln!("Number of lines in the final file   : {num_total_lines_final:>max_len$}\n");
        eprintln!("Total Run Time: {:?}\n", timer.get_elapsed_time());
    }
}
