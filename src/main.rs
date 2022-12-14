mod lib;
mod args;

use std::{
    fs::File,
    time::Instant,
    // https://doc.rust-lang.org/std/hash/index.html
    hash::{Hash, Hasher},
    io::{self, Read, BufReader, BufRead},
    collections::{HashMap, HashSet, hash_map::DefaultHasher},
};

use lib::*;
use clap::Parser;
use args::Arguments;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use ring::digest::{SHA256, SHA512};

// Inspiração: uniq, huniq e semiuniq.
// https://github.com/koraa/huniq/blob/main/src/main.rs
// https://github.com/kljensen/semiuniq

fn main() -> std::io::Result<()> {

    let time: Instant = Instant::now();

    // Parse command-line arguments 
    // https://docs.rs/clap/latest/clap/trait.Parser.html
    let arguments: Arguments = Arguments::parse();

    // The input is file or stdin.
    let input_file: Option<String> = arguments.file.clone();

    let mut reader: Box<dyn BufRead> = read_file_or_stdin(input_file);

    // Copies the entire contents of a reader into a writer.
    let mut writer: Vec<u8> = Vec::new();
    io::copy(&mut reader, &mut writer)?;

    // from_utf8() checks to ensure that the bytes are valid UTF-8
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

    // "Blake3", "Sha256", "Sha512" or "Hasher"
    let algorithm: &str = get_hash_algorithm(&arguments);
    let dispatch_table = make_dispatch_table();
    let hash_function = dispatch_table[algorithm];

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

        let hash: String = hash_function(&modified_line);

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

    test_csv_file(&string_utf8, &arguments);

    print_verbose(time, algorithm, &arguments, uniq_hashes, num_repeated_lines);

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

fn get_hash_algorithm(args: &Arguments) -> &'static str {
    if args.use_blake3 {
        "Blake3"
    }
    else if args.use_ring_sha256 {
        "Sha256"
    }
    else if args.use_ring_sha512 {
        "Sha512"
    } else {
        "Hasher"
    }
}

fn make_dispatch_table() -> HashMap<&'static str, fn(&str) -> String> {
    // https://stackoverflow.com/questions/51372702/how-do-i-make-a-dispatch-table-in-rust
    let mut dispatch_table: HashMap<&str, fn(&str) -> String> = HashMap::new();
    dispatch_table.insert("Blake3", blake3_hash);
    dispatch_table.insert("Sha256", |a| ring_hash(a, &SHA256));
    dispatch_table.insert("Sha512", |a| ring_hash(a, &SHA512));
    dispatch_table.insert("Hasher", |a| calculate_hash(a).to_string());
    dispatch_table
}

// https://doc.rust-lang.org/std/hash/index.html
fn calculate_hash<T>(input: &T) -> u64
    where T: Hash + ?Sized,
{
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

fn test_csv_file(all_lines: &str, args: &Arguments) {

    if args.test_csv_file {

        let mut delimiter_set: HashSet<usize> = HashSet::new();

        let csv_delimiter: char = args.csv_delimiter;

        for line in all_lines.lines() {
            let num_char: usize = line.count_char(csv_delimiter);
            delimiter_set.insert(num_char);
        }
        
        if delimiter_set.len() != 1 || delimiter_set.contains(&0) {
            println!();
            println!("Invalid CSV file!");
            println!("CSV column delimiter: '{csv_delimiter}'");
            println!("Column delimiter number observed in rows: {delimiter_set:?}");
        }
        else if args.verbose {
            // Get the first element from HashSet
            // first element = num_char
            // let vec: Vec<usize> = delimiter_set.into_iter().collect();
            let num_char: usize = delimiter_set.into_iter().next().unwrap();

            println!();
            println!("Valid CSV file!");
            println!("CSV column delimiter: '{csv_delimiter}'");
            println!("Column delimiter number observed in rows: {num_char}");
        }

        //println!("csv_file: {:?} ; csv_delimiter: '{}'", delimiter_set, ch);
    }
}

fn print_verbose(
    time: Instant,
    algorithm: &str,
    args: &Arguments,
    uniq_hashes: HashSet<String>,
    num_repeated_lines: usize
) {
    // cat file | wc -l
    let num_unique_lines: usize = uniq_hashes.len();
    let num_total_lines: usize = num_unique_lines + num_repeated_lines;
    let len = num_total_lines.to_string().len();

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