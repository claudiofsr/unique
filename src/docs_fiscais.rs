use crate::{format_number, NumberFormat};
use chrono::NaiveDate;
use serde::{self, de::Error, Deserialize, Deserializer, Serialize, Serializer};

const DATA_FORMAT: &str = "%d/%m/%Y";

// Adicionar #[serde(default)] no topo da struct. Isso diz ao Rust:
// "Se uma coluna não existir no arquivo, preencha o campo com o valor padrão (String vazia, 0 ou None)".

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DocsFiscais {
    #[serde(rename = "CNPJ do Contribuinte : NF Item (Todos)")]
    pub contribuinte_cnpj: String,

    #[serde(rename = "Nome do Contribuinte : NF Item (Todos)")]
    pub contribuinte_nome: String,

    #[serde(rename = "Entrada/Saída : NF (Todos)")]
    pub entrada_ou_saida: String,

    #[serde(rename = "CPF/CNPJ do Participante : NF (Todos)")]
    pub participante_cnpj: String,

    #[serde(rename = "Nome do Participante : NF (Todos)")]
    pub participante_nome: String,

    #[serde(rename = "CRT : NF (Todos)", deserialize_with = "string_as_opt_i64")]
    pub regime_tributario: Option<i64>,

    #[serde(rename = "Observações : NF (Todos)")]
    pub observacoes: String,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: CNPJ/CPF de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub remetente_cnpj1: String,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: CNPJ/CPF de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub remetente_cnpj2: String,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: Nome de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub remetente_nome: String,

    #[serde(
        rename = "CTe - Remetente das mercadorias transportadas: Município de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub remetente_municipio: String,

    #[serde(
        rename = "Descrição CTe - Indicador do 'papel' do tomador do serviço de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub tomador_papel1: String,

    #[serde(
        rename = "Descrição CTe - Indicador do 'papel' do tomador do serviço de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub tomador_papel2: String,

    #[serde(
        rename = "CTe - Outro tipo de Tomador: CNPJ/CPF de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub tomador_cnpj1: String,

    #[serde(
        rename = "CTe - Outro tipo de Tomador: CNPJ/CPF de Conhecimento : ConhecimentoInformacaoNFe"
    )]
    pub tomador_cnpj2: String,

    #[serde(
        rename = "CTe - UF do início da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub inicio_estado: String,

    #[serde(
        rename = "CTe - Nome do Município do início da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub inicio_municipio: String,

    #[serde(
        rename = "CTe - UF do término da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub termino_estado: String,

    #[serde(
        rename = "CTe - Nome do Município do término da prestação de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub termino_municipio: String,

    #[serde(
        rename = "CTe - Informações do Destinatário do CT-e: CNPJ/CPF de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub destinatario_cnpj: String,

    #[serde(
        rename = "CTe - Informações do Destinatário do CT-e: Nome de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub destinatario_nome: String,

    #[serde(
        rename = "CTe - Local de Entrega constante na Nota Fiscal: Nome de Conhecimento : ConhecimentoValoresPrestacaoServico-Componentes"
    )]
    pub local_entrega: String,

    #[serde(rename = "Descrição da Natureza da Operação : NF Item (Todos)")]
    pub descricao_natureza: String,

    #[serde(rename = "Cancelada : NF (Todos)")]
    pub cancelada: String,

    #[serde(rename = "Registro de Origem do Item : NF Item (Todos)")]
    pub origem: String,

    #[serde(rename = "Natureza da Base de Cálculo do Crédito Descrição : NF Item (Todos)")]
    pub natureza: String,

    #[serde(rename = "Modelo - Descrição : NF Item (Todos)")]
    pub modelo: String,

    #[serde(
        rename = "Número da Nota : NF Item (Todos)",
        deserialize_with = "string_as_opt_i64"
    )]
    pub num_doc: Option<i64>,

    #[serde(rename = "Chave da Nota Fiscal Eletrônica : NF Item (Todos)")]
    pub chave: String,

    #[serde(rename = "Inf. NFe - Chave de acesso da NF-e : ConhecimentoInformacaoNFe")]
    pub chave_de_acesso: String,

    #[serde(rename = "CTe - Observações Gerais de Conhecimento : ConhecimentoInformacaoNFe")]
    pub observacoes_gerais: String,

    #[serde(rename = "Dia da Emissão : NF Item (Todos)", with = "br_date_opt")]
    pub dia_emissao: Option<NaiveDate>,

    #[serde(rename = "Número da DI : NF Item (Todos)")]
    pub numero_di: String,

    #[serde(
        rename = "Número do Item : NF Item (Todos)",
        deserialize_with = "string_as_opt_i64"
    )]
    pub numero_item: Option<i64>,

    #[serde(
        rename = "Código CFOP : NF Item (Todos)",
        deserialize_with = "string_as_opt_i64"
    )]
    pub cfop: Option<i64>,

    #[serde(rename = "Descrição CFOP : NF Item (Todos)")]
    pub descricao_cfop: String,

    #[serde(rename = "Descrição da Mercadoria/Serviço : NF Item (Todos)")]
    pub descricao_mercadoria: String,

    #[serde(rename = "Código NCM : NF Item (Todos)")]
    pub ncm: String,

    #[serde(rename = "Descrição NCM : NF Item (Todos)")]
    pub descricao_ncm: String,

    #[serde(
        rename = "COFINS: Alíquota ad valorem - Atributo : NF Item (Todos)",
        deserialize_with = "string_as_opt_f64"
    )]
    pub aliq_cof_attr: Option<f64>,

    #[serde(
        rename = "PIS: Alíquota ad valorem - Atributo : NF Item (Todos)",
        deserialize_with = "string_as_opt_f64"
    )]
    pub aliq_pis_attr: Option<f64>,

    #[serde(rename = "CST COFINS Descrição : NF Item (Todos)")]
    pub cst_descricao_cof: String,

    #[serde(rename = "CST PIS Descrição : NF Item (Todos)")]
    pub cst_descricao_pis: String,

    #[serde(
        rename = "Valor Total : NF (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_total: Option<f64>,

    #[serde(
        rename = "Valor da Nota Proporcional : NF Item (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_item: Option<f64>,

    #[serde(
        rename = "Valor dos Descontos : NF Item (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_desconto: Option<f64>,

    #[serde(
        rename = "Valor Seguro : NF (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_seguro: Option<f64>,

    #[serde(
        rename = "COFINS: Valor do Tributo : NF Item (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_cof: Option<f64>,

    #[serde(
        rename = "PIS: Valor do Tributo : NF Item (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_pis: Option<f64>,

    #[serde(
        rename = "IPI: Valor do Tributo : NF Item (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_ipi: Option<f64>,

    #[serde(
        rename = "ISS: Base de Cálculo : NF Item (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_bc_iss: Option<f64>,

    #[serde(
        rename = "ISS: Valor do Tributo : NF Item (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_iss: Option<f64>,

    #[serde(
        rename = "ICMS: Alíquota : NF Item (Todos) NOISE OR",
        deserialize_with = "string_as_opt_f64"
    )]
    pub aliq_icms: Option<f64>,

    #[serde(
        rename = "ICMS: Base de Cálculo : NF Item (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_bc_icms: Option<f64>,

    #[serde(
        rename = "ICMS: Valor do Tributo : NF Item (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_icms: Option<f64>,

    #[serde(
        rename = "ICMS por Substituição: Valor do Tributo : NF Item (Todos) SOMA",
        deserialize_with = "string_as_opt_f64"
    )]
    pub valor_icms_sub: Option<f64>,
}

// --- DESERIALIZADORES PARA OPTION ---

/// Helper para identificar valores que devem ser tratados como nulos
fn is_null_val(s: &str) -> bool {
    // 1. Removemos espaços
    let s = s.trim();

    // 2. Definimos os padrões apenas em maiúsculo
    let null_patterns = ["", "<N/D>", "N/A", "*DIVERSOS*", "NULO", "NULL"];

    // 3. Verificamos se o valor normalizado está na lista
    null_patterns.iter().any(|&p| p.eq_ignore_ascii_case(s))
}

pub fn string_as_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(val) if !is_null_val(&val) => {
            let cleaned = format_number(&val, NumberFormat::Brazilian);
            cleaned
                .parse::<f64>()
                .map(Some)
                .map_err(|_| Error::custom(format!("Float inválido: '{}'", val)))
        }
        _ => Ok(None),
    }
}

pub fn string_as_opt_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(val) if !is_null_val(&val) => {
            let cleaned = val.trim().replace('.', "");
            cleaned
                .parse::<i64>()
                .map(Some)
                .map_err(|_| Error::custom(format!("Inteiro inválido: '{}'", val)))
        }
        _ => Ok(None),
    }
}

// --- DATAS (DD/MM/AAAA) ---

mod br_date_opt {
    use super::*;
    pub fn serialize<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => serializer.serialize_str(&d.format(DATA_FORMAT).to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) if !s.trim().is_empty() => NaiveDate::parse_from_str(s.trim(), DATA_FORMAT)
                .map(Some)
                .map_err(|_| Error::custom(format!("Data inválida: {}", s))),
            _ => Ok(None),
        }
    }
}

/// cargo test -- --show-output tests_docs_fiscais
#[cfg(test)]
mod tests_docs_fiscais {
    use super::*;
    use chrono::NaiveDate;
    use csv::ReaderBuilder;

    /// Teste para a lógica de identificação de valores nulos/vazios
    #[test]
    fn test_is_null_val() {
        assert!(is_null_val(""));
        assert!(is_null_val("  "));
        assert!(is_null_val("<N/D>"));
        assert!(is_null_val("n/a"));
        assert!(is_null_val("NULO"));
        assert!(is_null_val("*DIVERSOS*"));
        assert!(!is_null_val("123.45"));
        assert!(!is_null_val("352310..."));
    }

    /// Teste de integração: Simula uma linha de CSV completa para a struct DocsFiscais
    #[test]
    fn test_docs_fiscais_deserialization() {
        // Criamos um CSV com os headers exatamente como definidos no rename do Serde
        // Nota: Usamos ponto e vírgula como delimitador
        let data = "\
CNPJ do Contribuinte : NF Item (Todos);CRT : NF (Todos);Número da Nota : NF Item (Todos);Dia da Emissão : NF Item (Todos);Valor Total : NF (Todos) SOMA;ICMS: Alíquota : NF Item (Todos) NOISE OR
12.345.678/0001-99;3;123.456;25/12/2023;1.500,50;18,00";

        let mut reader = ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(true)
            .from_reader(data.as_bytes());

        let result = reader.deserialize::<DocsFiscais>().next().unwrap();

        assert!(
            result.is_ok(),
            "Falha ao deserializar DocsFiscais: {:?}",
            result.err()
        );
        let doc = result.unwrap();

        // Validação de strings simples
        assert_eq!(doc.contribuinte_cnpj, "12.345.678/0001-99");

        // Validação de Option<i64> (com remoção de ponto de milhar)
        assert_eq!(doc.regime_tributario, Some(3));
        assert_eq!(doc.num_doc, Some(123456));

        // Validação de Option<NaiveDate>
        assert_eq!(
            doc.dia_emissao,
            Some(NaiveDate::from_ymd_opt(2023, 12, 25).unwrap())
        );

        // Validação de Option<f64> (formato brasileiro)
        assert_eq!(doc.valor_total, Some(1500.50));
        assert_eq!(doc.aliq_icms, Some(18.00));
    }

    /// Teste específico para casos de erro e valores nulos no CSV
    #[test]
    fn test_docs_fiscais_null_and_edge_cases() {
        let data = "\
CNPJ do Contribuinte : NF Item (Todos);CRT : NF (Todos);Número da Nota : NF Item (Todos);Dia da Emissão : NF Item (Todos);Valor Total : NF (Todos) SOMA
Empresa Teste;N/A;<N/D>; ;NULO";

        let mut reader = ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(true)
            .from_reader(data.as_bytes());

        let doc: DocsFiscais = reader.deserialize().next().unwrap().unwrap();

        // Todos os campos numéricos e datas devem resultar em None para os padrões de null_patterns
        assert_eq!(doc.regime_tributario, None);
        assert_eq!(doc.num_doc, None);
        assert_eq!(doc.dia_emissao, None);
        assert_eq!(doc.valor_total, None);
    }

    /// Teste isolado para o conversor de data BR
    #[test]
    fn test_br_date_deserializer() {
        use serde_json;

        // Simulando como o Serde veria uma string de data
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct DateWrapper {
            #[serde(with = "br_date_opt")]
            date: Option<NaiveDate>,
        }

        // Caso Sucesso
        let json = r#"{"date": "31/01/2024"}"#;
        let decoded: DateWrapper = serde_json::from_str(json).unwrap();
        assert_eq!(
            decoded.date,
            Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap())
        );

        // Caso Vazio
        let json_empty = r#"{"date": ""}"#;
        let decoded_empty: DateWrapper = serde_json::from_str(json_empty).unwrap();
        assert_eq!(decoded_empty.date, None);

        // Caso Inválido (Deve retornar Erro)
        let json_err = r#"{"date": "31-01-2024"}"#;
        let res: Result<DateWrapper, _> = serde_json::from_str(json_err);
        assert!(res.is_err());
    }
}
