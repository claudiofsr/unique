// command-line arguments
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Arguments {

    /// Read lines from file (or standard input)
    #[clap(required = false)]
    pub file: Option<String>,

    /// Ignore differences in case when comparing lines
    #[clap(short('i'), long("ignore_case"), value_parser, default_value_t = false)]
    pub ignore_case: bool,

    /// Returns lines with leading and trailing whitespace removed
    #[clap(short('t'), long("trim"), value_parser, default_value_t = false)]
    pub trim_line: bool,

    /// Replace multiple whitespace with just one
    #[clap(short('w'), long("whitespace"), value_parser, default_value_t = false)]
    pub remove_multiple_whitespace: bool,

    /// Use ring Sha256 algorithm to hash the lines
    #[clap(short('2'), long("sha256"), value_parser, default_value_t = false)]
    pub use_ring_sha256: bool,

    /// Use ring Sha512 algorithm to hash the lines.
    /// Less chance of collisions.
    #[clap(short('5'), long("sha512"), value_parser, default_value_t = false)]
    pub use_ring_sha512: bool,

    /// Test valid CSV file.
    /// All lines must have the same number of columns.
    /// Columns are identified according to the delimiter character.
    #[clap(short('c'), long("csv"), value_parser, default_value_t = false)]
    pub test_csv_file: bool,

    /// Set the CSV character separator/delimiter to:
    /// comma ','
    /// semicolon ';'
    /// pipe '|'
    /// or ...
    #[clap(short('s'), long("separator"), value_parser, default_value_t = ';', required = false, requires("test_csv_file"))]
    pub csv_separator: char,

    /// Only print repeated lines
    #[clap(short('r'), long("repeated"), value_parser, default_value_t = false)]
    pub only_print_repeated_lines: bool,

    /// Show number of unique, repeated and total lines
    #[clap(short('v'), long("verbose"), value_parser, default_value_t = false, requires("file"))]
    pub verbose: bool, 
}
