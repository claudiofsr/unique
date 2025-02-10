// command-line arguments
use clap::Parser;
use std::path::PathBuf;

// https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help
fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .usage(
            anstyle::Style::new()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan)))
                .bold(),
        )
        .header(
            anstyle::Style::new()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan)))
                .bold()
                .underline(),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
}

#[derive(Parser, Debug, Clone)]
#[command(
    // Read from `Cargo.toml`
    author, version, about,
    long_about = None,
    next_line_help = true,
    styles=get_styles(),
)]
pub struct Arguments {
    /// FILE input (or standard input).
    #[arg(required = false)]
    pub file: Option<PathBuf>,

    /// Remove empty lines.
    #[arg(short('e'), long("empty_lines"), default_value_t = false)]
    pub remove_empty_lines: bool,

    /// Ignore differences in case when comparing lines.
    #[arg(short('i'), long("ignore_case"), default_value_t = false)]
    pub ignore_case: bool,

    /// Returns lines with leading and trailing whitespace removed.
    #[arg(short('t'), long("trim"), default_value_t = false)]
    pub trim_line: bool,

    /// Replace multiple whitespace with just one.
    #[arg(short('w'), long("whitespace"), default_value_t = false)]
    pub replace_multiple_whitespaces: bool,

    /// Parse valid CSV file.
    /// All lines must have the same number of columns.
    /// Columns are identified according to the delimiter character.
    /// The default delimiter character is ';'.
    #[arg(short('c'), long("csv"), default_value_t = false, verbatim_doc_comment)]
    pub parse_csv_file: bool,

    /// Set the field separator (delimiter) for CSV files to:
    /// comma ','
    /// semicolon ';'
    /// pipe '|'
    /// or another char.
    #[arg(short('s'), long("separator"), default_value_t = ';', required = false)]
    #[arg(requires("parse_csv_file"), verbatim_doc_comment)]
    pub separator: char,

    /// Formats the date in %d/%m/%Y format in CSV files.
    /// Example:
    /// "1 / 1 / 2023" => "01/01/2023"
    #[arg(
        short('d'),
        long("format_date"),
        default_value_t = false,
        required = false
    )]
    #[arg(requires("parse_csv_file"), verbatim_doc_comment)]
    pub format_date: bool,

    /// Format 44-digit key and calculate key check digit in CSV files.
    #[arg(
        short('k'),
        long("format_key"),
        default_value_t = false,
        required = false
    )]
    #[arg(requires("parse_csv_file"), verbatim_doc_comment)]
    pub format_key: bool,

    /// Format numbers to float64 in CSV files.
    /// Example:
    /// 34.542.675,01 => 34542675.01
    /// 34,542,675.01 => 34542675.01
    #[arg(
        short('n'),
        long("format_number"),
        default_value_t = false,
        required = false
    )]
    #[arg(requires("parse_csv_file"), verbatim_doc_comment)]
    pub format_number: bool,

    /// Print only duplicate or repeated lines.
    #[arg(short('r'), long("repeated"), default_value_t = false)]
    pub only_print_repeated_lines: bool,

    /// Show number of unique, repeated and total lines.
    #[arg(short('v'), long("verbose"), default_value_t = false)]
    pub verbose: bool,
}
