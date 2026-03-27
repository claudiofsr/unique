use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::{
    fs,
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
};

use crate::{UniqueError, UniqueResult};

/// Abre o arquivo ou Stdin retornando um Result.
pub fn read_file_or_stdin(path: &Option<PathBuf>) -> UniqueResult<Box<dyn BufRead>> {
    match path {
        Some(filename) => {
            // Usamos .map_err para converter o std::io::Error em UniqueError::FileError
            // e o operador '?' para retornar o erro precocemente se ele ocorrer.
            let file = fs::OpenOptions::new()
                .read(true)
                .write(false)
                .create(false)
                .open(filename)
                .map_err(|e| UniqueError::FileError {
                    path: filename.display().to_string(),
                    source: e,
                })?;

            Ok(Box::new(BufReader::new(file)))
        }
        None => {
            // Para o stdin, o erro de IO (se houver) será convertido
            // automaticamente para UniqueError::Io devido ao #[from]
            Ok(Box::new(BufReader::new(io::stdin())))
        }
    }
}

/// Converte bytes em String, tentando UTF-8 e Windows-1252.
/// Propaga os erros em caso de falha total, sem dar panic.
pub fn get_string_utf8_from_slice_bytes(slice_bytes: &[u8]) -> UniqueResult<String> {
    // 1. Limpeza eficiente de bytes de nova linha
    let mut vec_bytes = slice_bytes.to_vec();
    vec_bytes.retain(|&byte| byte != b'\r' && byte != b'\n');

    // 2. Tentativa 1: UTF-8
    match std::str::from_utf8(&vec_bytes) {
        Ok(valid_str) => Ok(valid_str.to_string()),
        Err(error1) => {
            // 3. Tentativa 2: Fallback para Windows-1252
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(vec_bytes.as_slice());

            let mut buffer = String::new();
            match decoder.read_to_string(&mut buffer) {
                Ok(_) => Ok(buffer),
                Err(error2) => {
                    // 4. Falha total: Informa ambos os erros através do enum
                    Err(UniqueError::DecodingFallback {
                        utf8_err: error1.to_string(),
                        fallback_err: error2.to_string(),
                    })
                }
            }
        }
    }
}
