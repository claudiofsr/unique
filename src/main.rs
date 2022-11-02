mod lib;
mod args;

use std::{
    fs::File,
    time::Instant,
    // https://doc.rust-lang.org/std/hash/index.html
    hash::{Hash, Hasher},
    io::{self, Read, BufReader, BufRead},
    collections::{HashSet, hash_map::DefaultHasher},
};

use lib::*;
use clap::Parser;
use args::Arguments;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use ring::digest::{SHA256, SHA512};

// Origem: os dois projetos seguintes: huniq e semiuniq
// https://github.com/koraa/huniq/blob/main/src/main.rs
// https://github.com/kljensen/semiuniq

fn main() -> std::io::Result<()> {

    let time: Instant = Instant::now();

    // Parse command-line arguments 
    // https://docs.rs/clap/latest/clap/trait.Parser.html
    let arguments: Arguments = Arguments::parse();

    // The input is file or stdin.
    let input_file: Option<String> = arguments.file;

    let mut reader: Box<dyn BufRead> = read_file_or_stdin(input_file);

    // Copies the entire contents of a reader into a writer.
    let mut writer: Vec<u8> = Vec::new();
    io::copy(&mut reader, &mut writer)?;

    let string_utf8: String = match std::str::from_utf8(&writer) {
        Ok(str) => str.to_string(),
        Err(_) => {
            let mut data = DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1252))
            .build(writer.as_slice());

            let mut buffer = String::new();
            let _number_of_bytes = data.read_to_string(&mut buffer)
            .expect("\nProblem reading data from file in buffer!\nConvert data to UTF-8!\n");
            buffer
        }
    };

    let mut uniq_hashes: HashSet<String> = HashSet::new();
    let mut num_repeated_lines: usize = 0;

    for line in string_utf8.lines() {

        let mut modified_line: String = line.to_string();

        if arguments.ignore_case {
            modified_line = modified_line.to_lowercase();
        }
    
        if arguments.trim_line {
            modified_line = modified_line.trim().to_string();
        }

        if arguments.remove_multiple_whitespace {
            modified_line = modified_line.remove_multiple_whitespace();
        }

        let hash: String = if arguments.use_ring_sha256 {
            ring_hash(&modified_line, &SHA256)
        }
        else if arguments.use_ring_sha512 {
            ring_hash(&modified_line, &SHA512)
        } else {
            calculate_hash(&modified_line).to_string()
        };

        if uniq_hashes.insert(hash) {
            if !arguments.only_print_repeated_lines {
                println!("{line}");
            }
        } else {
            if arguments.only_print_repeated_lines {
                println!("{line}");
            }
            num_repeated_lines += 1;
        }
    }

    test_csv_file(&string_utf8);

    print_verbose(time, uniq_hashes, num_repeated_lines);

    Ok(())
}

fn read_file_or_stdin(input_file: Option<String>) -> Box<dyn BufRead> {
    // If we don't receive an input file, use stdin.
    let reader: Box<dyn BufRead> = match input_file {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => {
            let file = match File::open(&filename) {
                Ok(f) => f,
                Err(why) => panic!(":\nProblem opening the file: \"{filename}\"\n{why:?}\n"),
            };
            Box::new(BufReader::new(file))
        }
    };
    reader
}

// https://doc.rust-lang.org/std/hash/index.html
fn calculate_hash<T>(input: &T) -> u64
where T: Hash + ?Sized,
{
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

fn test_csv_file(all_lines: &str) {

    let args: Arguments = Arguments::parse();

    if args.test_csv_file {

        let mut count_separator: HashSet<usize> = HashSet::new();

        let csv_separator: char = args.csv_separator;

        for line in all_lines.lines() {
            let num_char: usize = line.count_char(csv_separator);
            count_separator.insert(num_char);
        }
        
        if count_separator.len() != 1 || count_separator.contains(&0) {
            println!();
            println!("Invalid CSV file!");
            println!("CSV column separator: '{csv_separator}'");
            println!("Column separator number observed in rows: {count_separator:?}\n");
        }

        //println!("csv_file: {:?} ; csv_separator: '{}'", count_separator, ch);
    }
}

fn print_verbose(time: Instant, uniq_hashes: HashSet<String>, num_repeated_lines: usize) {
    // cat file | wc -l
    let num_unique_lines: usize = uniq_hashes.len();
    let num_total_lines: usize = num_unique_lines + num_repeated_lines;
    let len = num_total_lines.to_string().len();

    let args: Arguments = Arguments::parse();

    let algorithm: &str = if args.use_ring_sha256 {
        "ring Sha256"
    }
    else if args.use_ring_sha512 {
        "ring Sha512"
    } else {
        "DefaultHasher"
    };

    // Show number of unique, repeated and total lines

    if args.verbose {
        println!();
        println!("Number of unique lines  : {num_unique_lines:>len$}");
        println!("Number of repeated lines: {num_repeated_lines:>len$}");
        println!("Number of total lines   : {num_total_lines:>len$}");
        println!();
        println!("Algorithm to hash the lines: {algorithm}");
        println!("Total Run Time: {:?}",time.elapsed());
    }
}