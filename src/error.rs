use thiserror::Error;

pub type UniqueResult<T> = Result<T, UniqueError>;

#[derive(Error, Debug)]
pub enum UniqueError {
    #[error("Erro de E/S (IO): {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro no processamento de CSV: {0}")]
    Csv(#[from] csv::Error),

    #[error("Falha na conversão UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Erro de mapeamento de colunas: {0}")]
    Mapping(String),

    #[error("Falha ao converter '{0}' para número (f64)")]
    ParseFloat(String),

    #[error("Erro desconhecido: {0}")]
    Unknown(String),
}
