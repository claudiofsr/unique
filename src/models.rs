use crate::UniqueResult;

// Alias opcional para simplificar a assinatura da função
pub type AnalysisResult = UniqueResult<Vec<Option<AnalyzedLine>>>;

/// Representa o resultado detalhado da análise de uma única linha.
///
/// Esta struct armazena metadados necessários para a contagem final
/// e para manter a ordem de processamento no Rayon.
#[derive(Debug, Clone)]
pub struct AnalyzedLine {
    /// O número original da linha no arquivo de entrada.
    pub line_number: usize,
    /// O conteúdo da linha após todas as transformações e limpezas.
    pub content: String,
    /// A contagem de colunas detectadas (útil para validação de CSV).
    pub column_count: usize,
    /// Indica se a linha estava vazia antes ou após o processamento.
    pub is_empty: bool,
}

impl AnalyzedLine {
    /// Cria uma nova instância de uma linha vazia.
    pub fn empty(line_number: usize) -> Self {
        Self {
            line_number,
            content: String::new(),
            column_count: 0,
            is_empty: true,
        }
    }
}
