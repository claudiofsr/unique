use crate::{
    format_date, format_key, format_number, Arguments, DocsFiscais, UniqueError, UniqueResult,
};
use claudiofsr_lib::StrExtension;
use csv::{ReaderBuilder, WriterBuilder};

/// Centraliza a lógica de formatação de colunas.
pub fn apply_formatting(col: &str, args: &Arguments) -> String {
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

/// Helper para serializar dados garantindo as mesmas configurações da analise_line original.
/// Nota: O csv::Writer adiciona um '\n' ao final.
fn serialize_to_string<T: serde::Serialize>(data: T) -> UniqueResult<String> {
    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .flexible(false)
        .from_writer(vec![]);

    wtr.serialize(data)?;

    let bytes = wtr.into_inner().map_err(|e| e.into_error())?;
    Ok(String::from_utf8(bytes)?)
}

/// Realiza a análise de linha (Modo Simples).
pub fn analise_line(args: &Arguments, line: &str) -> UniqueResult<(String, usize)> {
    let mut modified_line = line.to_owned();
    let mut num_cols = 0;

    if args.parse_csv_file {
        let mut reader = ReaderBuilder::new()
            .quoting(true)
            .double_quote(true)
            .has_headers(false)
            .trim(csv::Trim::All)
            .flexible(false)
            .delimiter(args.separator as u8)
            .from_reader(line.as_bytes());

        if let Some(result) = reader.records().next() {
            let record = result?;
            num_cols = record.len();

            let formatted_cols: Vec<String> = record
                .iter()
                .map(|col| apply_formatting(col, args))
                .collect();

            modified_line = serialize_to_string(&formatted_cols)?;
        }
    }

    Ok((post_process_string(modified_line, args), num_cols))
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
    Ok((post_process_string(modified_line, args), num_cols))
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

/// Aplica limpeza final de espaços e trim conforme argumentos da CLI.
pub fn post_process_string(mut s: String, args: &Arguments) -> String {
    if args.replace_multiple_whitespaces {
        s = s.replace_multiple_whitespaces();
    }
    if args.trim_line {
        s = s.trim().to_string();
    }
    s
}
