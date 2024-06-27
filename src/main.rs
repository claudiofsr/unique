use args::Arguments;
use clap:: Parser;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use claudiofsr_lib::StrExtension;
use rayon::prelude::*;

use std::{
    path::PathBuf,
    time::Instant,
    fs::{self, File},
    io::{
        self,
        Read,
        BufReader,
        BufRead
    },
    collections::HashSet,
};

/**
Inspiração: uniq, huniq e semiuniq.
https://github.com/koraa/huniq/blob/main/src/main.rs
https://github.com/kljensen/semiuniq

clear && cargo test -- --nocapture
clear && cargo run -- -h
cargo b -r && cargo install --path=.
*/

// Functions defined in lib.rs
use unique::*;

const CHUNK_SIZE: usize = 10_000;
const NEWLINE_BYTE: u8 = b'\n';

fn main() -> MyResult<()> {
    let time: Instant = Instant::now();
    let arguments: Arguments = Arguments::parse();

    // The input is file or stdin.
    let input_file: Option<PathBuf> = arguments.file.clone();
    let mut buffer: Box<dyn BufRead> = read_file_or_stdin(&input_file);

    let mut delimiter_set: HashSet<usize> = HashSet::new();
    let mut uniq_hashes: HashSet<String> = HashSet::new();
    let mut num_repeated_lines: usize = 0;
    let mut num_empty_lines: usize = 0;

    // Evitar ler todo o arquivo na memória
    // Ler apenas uma linha de cada vez do Buffer para análise

    let mut line_number: usize = 0;
    let mut num_bytes: usize = 1;

    while num_bytes > 0 {
        // Vec<line_number, line>
        let mut vec_lines: Vec<(usize, Vec<u8>)> = Vec::new();

        loop {
            let mut line: Vec<u8> = Vec::new();
            num_bytes = buffer.read_until(NEWLINE_BYTE, &mut line)?;
            if num_bytes == 0 { // EOF is reached
                break;
            }
            line_number += 1;
            vec_lines.push((line_number, line));
            if vec_lines.len() >= CHUNK_SIZE {
                break;
            }
        }

        let vector: MyResult<Vec<(usize, String, usize, bool)>> = vec_lines
            .into_par_iter() // rayon: parallel iterator
            .map(|(line_number, vec_bytes)| {
                let line_utf8: String = get_string_utf8_from_slice_bytes(&vec_bytes);
                let (new_line, num_cols) = analise_line(&arguments, &line_utf8, line_number)?;
                let empty_lines: bool = line_utf8.trim().is_empty();
                Ok((line_number, new_line, num_cols, empty_lines))
            })
            .collect();

        // Não é necessário ordenar, pois rayon coleta já ordenado!
        // No sorting required as rayon collects already sorted!
        // vector.par_sort_by_key(|&(line_number,..)| line_number);

        for tuple in vector? {
            let (_line_number, line, num_cols, empty_lines) = tuple;
            // convert a boolean to an integer
            num_empty_lines += empty_lines as usize;
            apply_analysis(&arguments, &line, num_cols, &mut num_repeated_lines, &mut uniq_hashes, &mut delimiter_set);
        }
    }

    analise_csv_file(&arguments, delimiter_set);

    print_verbose(&arguments, time, uniq_hashes, num_repeated_lines, num_empty_lines);

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
        },
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
                },
            };

            buffer
        }
    };

    // remove new line: "\r\n" or "\n"
    // string_utf8.trim_end_matches(&['\r','\n']).to_string()
    string_utf8
}

fn analise_line(args: &Arguments, line: &str, _line_number: usize) -> MyResult<(String, usize)> {
    let mut modified_line: String = line.to_owned();
    let mut num_cols: usize = 0;

    if args.parse_csv_file {
 
        //let perdcomps: Vec<PerDcomp> = parse_csv_with_serde(&modified_line, args, line_number)?;
        //for perdcomp in perdcomps { println!("perdcomp: {perdcomp:?}") }

        let cols: Vec<String> = parse_csv_line(&modified_line, args);

        // https://docs.rs/csv/latest/csv/struct.WriterBuilder.html
        // Add double quotes when necessary
        let mut wtr = WriterBuilder::new()
            //.delimiter(args.separator as u8)
            .delimiter(b';')
            .has_headers(false)
            .flexible(false)
            .from_writer(vec![]);
        wtr.write_record(&cols)?;
        modified_line = String::from_utf8(wtr.into_inner()?)?;
        
        num_cols = cols.len();

        //modified_line = cols.join(&args.separator.to_string());

        /*
        let perdcomps: Vec<PerDcomp> = parse_csv_with_serde(&modified_line, args, line_number)?;

        // When writing records with Serde using structs, 
        // the header row is written automatically.

        let mut wtr = WriterBuilder::new()
            //.delimiter(args.separator as u8)
            .delimiter(b';')
            .has_headers(false)
            .flexible(false)
            .from_writer(vec![]);

        for perdcomp in &perdcomps { 
            //println!("perdcomp: {perdcomp:?}");
            if line_number == 1 {
                continue;
            }
            wtr.serialize(perdcomp)?;
        }

        wtr.flush()?;

        num_cols = perdcomps.len();
        */

        //println!("num_cols: {num_cols} ; cols: {cols:?} ; modified_line: {modified_line:?}\n");
    }

    if args.replace_multiple_whitespaces {
        modified_line = modified_line.replace_multiple_whitespaces();
    }

    if args.trim_line {
        modified_line = modified_line.trim().to_string();
    }

    Ok((modified_line, num_cols))
}

fn parse_csv_line(line: &str, args: &Arguments) -> Vec<String> {
    let mut reader = ReaderBuilder::new()
        .quoting(true)
        .double_quote(true)
        .has_headers(false)
        .trim(csv::Trim::All)
        .flexible(false)
        .delimiter(args.separator as u8)
        .from_reader(line.as_bytes());

    let sep: char = args.separator;
    //println!("reader {reader:?}\n");

    let records: Vec<StringRecord> = reader
        .records()
        .filter_map(|result| result.ok())
        .collect();

    // Vec<StringRecord> to Vec<&str>
    let cols: Vec<String> = records
        .first()
        .map(|cols| cols
            .into_iter()
            .map(|col| col.replace("\\n", " ").trim().to_string()) // replace new_line by " "
            .map(|col| col.replace(sep, "-").to_string()) // replace separator ';' by '-'
            .map(|col| if args.format_date { format_date(col) } else { col } )
            .map(|col| if args.format_key { format_key(col) } else { col } )
            .map(|col| if args.format_number { format_number(col) } else { col } )
            .collect()
        )
        .unwrap_or_default();

    cols
}

// https://github.com/BurntSushi/rust-csv
#[allow(dead_code)]
fn parse_csv_with_serde(line: &str, args: &Arguments, line_number: usize,) -> MyResult<Vec<PerDcomp>> {
    let mut reader = ReaderBuilder::new()
        .quoting(true)
        .double_quote(true)
        .has_headers(false)
        .trim(csv::Trim::All)
        .flexible(false)
        .delimiter(args.separator as u8)
        .from_reader(line.as_bytes());

    //println!("reader {reader:?}\n");

    let mut records: Vec<PerDcomp> = Vec::new();

    for result in reader.deserialize() {
        if line_number == 1 {
            continue;
        }
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: PerDcomp = result?;
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
        }
        else if args.verbose {
            let first_element = delimiter_vec[0];
            eprintln!();
            eprintln!("Valid CSV file!");
            eprintln!("CSV column delimiter: '{separator}'");
            eprintln!("Column delimiter number observed in rows: {first_element}");
        }
    }
}

fn print_verbose(
    args: &Arguments,
    time: Instant,
    uniq_hashes: HashSet<String>,
    num_repeated_lines: usize,
    num_empty_lines: usize,
) {
    // cat file | wc -l
    let num_unique_lines = uniq_hashes.len();
    let num_total_lines_original = num_unique_lines + num_repeated_lines;
    let max_len = num_total_lines_original.to_string().len();

    let num_total_lines_final = if args.remove_empty_lines && num_empty_lines > 0 {
        num_unique_lines - 1
    } else {
        num_unique_lines
    };

    // Show number of unique, repeated and total lines
    // eprintln! macro prints to the standard error stream

    if args.verbose {
        eprintln!();
        eprintln!("Number of lines in the original file: {num_total_lines_original:>max_len$}");
        eprintln!("Number of unique lines              : {num_unique_lines:>max_len$}");
        eprintln!("Number of repeated lines            : {num_repeated_lines:>max_len$}");
        eprintln!("Number of empty lines               : {num_empty_lines:>max_len$}");
        eprintln!("Number of lines in the final file   : {num_total_lines_final:>max_len$}\n");
        eprintln!("Total Run Time: {:?}\n",time.elapsed());
    }
}