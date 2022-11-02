// command-line arguments
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
#[command(next_line_help = true)]
pub struct Arguments {

    /// Read lines from FILE (or standard input)
    #[arg(required = false)]
    pub file: Option<String>,

    /// Ignore differences in case when comparing lines
    #[arg(short('i'), long("ignore_case"), default_value_t = false)]
    pub ignore_case: bool,

    /// Returns lines with leading and trailing whitespace removed
    #[arg(short('t'), long("trim"), default_value_t = false)]
    pub trim_line: bool,

    /// Replace multiple whitespace with just one
    #[arg(short('w'), long("whitespace"), default_value_t = false)]
    pub remove_multiple_whitespace: bool,

    /// Use ring Sha512 algorithm to hash the lines.
    #[arg(short('s'), long("sha512"), default_value_t = false)]
    pub use_ring_sha512: bool,

    /// Use blake3 algorithm to hash the lines
    #[arg(short('b'), long("blake3"), default_value_t = false)]
    pub use_blake3: bool,

    /// Test valid CSV file.
    /// All lines must have the same number of columns.
    /// Columns are identified according to the delimiter character.
    /// The default delimiter character is ';'.
    #[arg(short('c'), long("csv"), default_value_t = false, verbatim_doc_comment)]
    pub test_csv_file: bool,

    /// Set the CSV character separator/delimiter to:
    /// comma ','
    /// semicolon ';'
    /// pipe '|'
    /// or another char.
    #[arg(short('d'), long("delimiter"), default_value_t = ';', required = false)]
    #[arg(requires("test_csv_file"), verbatim_doc_comment)]
    pub csv_delimiter: char,

    /// Only print repeated lines
    #[arg(short('r'), long("repeated"), default_value_t = false)]
    pub only_print_repeated_lines: bool,

    /// Show number of unique, repeated and total lines
    #[arg(short('v'), long("verbose"), default_value_t = false, requires("file"))]
    pub verbose: bool, 
}
