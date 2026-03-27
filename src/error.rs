use thiserror::Error;

pub type UniqueResult<T> = Result<T, UniqueError>;

#[derive(Error, Debug)]
pub enum UniqueError {
    // 1. Variante para erros de IO genéricos (como o read_until no stdin)
    // O #[from] funciona aqui porque é uma tupla de um único elemento
    #[error("Erro de E/S (IO): {0}")]
    Io(#[from] std::io::Error),

    // 2. Variante específica para quando temos um caminho de arquivo
    #[error("Erro no arquivo '{path}': {source}")]
    FileError {
        path: String,
        source: std::io::Error,
    },

    #[error("Erro no processamento de CSV: {0}")]
    Csv(#[from] csv::Error),

    #[error("Falha na conversão UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Falha na decodificação (UTF-8: {utf8_err}, Fallback: {fallback_err})")]
    DecodingFallback {
        utf8_err: String,
        fallback_err: String,
    },

    #[error("Erro de mapeamento de colunas: {0}")]
    Mapping(String),

    #[error("Falha ao converter '{0}' para número (f64)")]
    ParseFloat(String),

    #[error("Erro desconhecido: {0}")]
    Unknown(String),
}
