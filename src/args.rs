// command-line arguments
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum NumberFormat {
    #[default]
    Brazilian, // Milhar: '.' , Decimal: ','
    International, // Milhar: ',' , Decimal: '.'
}

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
    /// Parse CSV specifically using the DocsFiscais mapping.
    /// This ensures high robustness by validating column names and data types.
    #[arg(short('m'), long, requires("parse_csv_file"), verbatim_doc_comment)]
    pub map_docs_fiscais: bool,

    /// FILE input (or standard input if empty).
    pub file: Option<PathBuf>,

    /// Remove empty lines.
    #[arg(short('e'), long)]
    pub remove_empty_lines: bool,

    /// Ignore differences in case when comparing lines.
    #[arg(short('i'), long)]
    pub ignore_case: bool,

    /// Returns lines with leading and trailing whitespace removed.
    #[arg(short('t'), long)]
    pub trim_line: bool,

    /// Replace multiple whitespace with just one.
    #[arg(short('w'), long)]
    pub replace_multiple_whitespaces: bool,

    /// Parse valid CSV file.
    /// All lines must have the same number of columns based on the separator.
    #[arg(short('c'), long("csv"), verbatim_doc_comment)]
    pub parse_csv_file: bool,

    /// Set the field separator (delimiter) for CSV files to:
    /// comma ','
    /// semicolon ';'
    /// pipe '|'
    /// or another char.
    #[arg(
        short('s'),
        long("separator"),
        default_value_t = ';',
        requires("parse_csv_file")
    )]
    pub separator: char,

    /// Formats the date in %d/%m/%Y format in CSV files.
    /// Example:
    /// "1 / 1 / 2023" => "01/01/2023"
    #[arg(short('d'), default_value_t = false, requires("parse_csv_file"))]
    pub format_date: bool,

    /// Format 44-digit NFe key and validates check digit in CSV files.
    #[arg(short('k'), long, requires("parse_csv_file"))]
    pub format_key: bool,

    /// Format numeric strings to computational float (f64) in CSV files.
    ///
    /// Example:
    /// 34.542.675,01 => 34542675.01
    /// 34,542,675.01 => 34542675.01
    #[arg(short('f'), long, requires("parse_csv_file"), verbatim_doc_comment)]
    pub format_number: bool,

    /// Choose the number format for parsing decimals and thousands.
    #[arg(
        short('n'),
        long,
        value_enum,
        default_value_t = NumberFormat::Brazilian,
        requires("format_number"),
        verbatim_doc_comment
    )]
    pub number_format: NumberFormat,

    /// Print only duplicate or repeated lines.
    #[arg(short('r'), long("repeated"))]
    pub only_print_repeated_lines: bool,

    /// Show number of unique, repeated and total lines.
    #[arg(short('v'), long)]
    pub verbose: bool,
}
