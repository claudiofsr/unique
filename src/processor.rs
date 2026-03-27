use crate::{
    format_date, format_key, format_number, Arguments, DocsFiscais, UniqueError, UniqueResult,
};
use claudiofsr_lib::StrExtension;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};

/// Simple line analysis: iterates through columns and applies formatting.
pub fn analise_line(line: &str, args: &Arguments) -> UniqueResult<(String, usize)> {
    let mut reader = ReaderBuilder::new()
        .delimiter(args.separator as u8)
        .has_headers(false)
        .from_reader(line.as_bytes());

    if let Some(result) = reader.records().next() {
        let record = result?;
        let num_cols = record.len();

        // Apply formatting to each column
        let formatted_cols: Vec<String> = record
            .iter()
            .map(|col| apply_formatting(col, args))
            .collect();

        let modified = serialize_to_string(&formatted_cols)?;
        return Ok((post_process_string(modified, args), num_cols));
    }

    Ok((line.to_string(), 0))
}

/// Robust line analysis using the `DocsFiscais` struct mapping via Serde.
///
/// This function uses a pre-parsed `header_record` to map CSV columns to
/// struct fields by name. It is significantly faster than creating
/// virtual CSV strings for every line.
pub fn analise_line_with_serde(
    line: &str,
    header_record: &Option<StringRecord>,
    args: &Arguments,
) -> UniqueResult<(String, usize)> {
    // 1. Parse the raw line into a CSV StringRecord
    let mut reader = ReaderBuilder::new()
        .quoting(true)
        .double_quote(true)
        .has_headers(false)
        .trim(csv::Trim::All)
        .delimiter(args.separator as u8)
        .from_reader(line.as_bytes());

    let record = match reader.records().next() {
        Some(res) => res?,
        None => return Ok((String::new(), 0)),
    };

    let num_cols = record.len();

    // 2. Deserialize using the header for field mapping
    // We pass the header as context so Serde knows which column is which
    let doc: DocsFiscais = record
        .deserialize(header_record.as_ref())
        .map_err(|e| UniqueError::Mapping(format!("Serde mapping failed: {}", e)))?;

    // 3. Serialize the struct back to a clean semicolon-separated string
    let processed_line = serialize_to_string(&doc)?;
    Ok((post_process_string(processed_line, args), num_cols))
}

/// Applies all enabled formatting rules to a single CSV cell.
///
/// This is the central point for data transformation based on CLI arguments.
fn apply_formatting(col: &str, args: &Arguments) -> String {
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
}

/// Helper to serialize a record or struct into a semicolon-delimited String.
fn serialize_to_string<T: serde::Serialize>(data: &T) -> UniqueResult<String> {
    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .flexible(false)
        .from_writer(vec![]);

    wtr.serialize(data)?;

    let bytes = wtr.into_inner().map_err(|e| e.into_error())?;
    Ok(String::from_utf8(bytes)?.trim_end().to_string())
}

/// Final cleanup: handles whitespace replacement and trimming.
fn post_process_string(mut s: String, args: &Arguments) -> String {
    if args.replace_multiple_whitespaces {
        s = s.replace_multiple_whitespaces();
    }
    if args.trim_line {
        s = s.trim().to_string();
    }
    s
}
