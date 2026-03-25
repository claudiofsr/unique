pub mod args;
mod error;
pub mod structures;

pub use args::*;
pub use error::{UniqueError, UniqueResult};
pub use structures::*;

use chrono::NaiveDate;
use regex::Regex;
use std::ops::Deref;
use std::sync::LazyLock as Lazy;

pub use claudiofsr_lib::StrExtension;

pub const DIAS: [u16; 31] = {
    let mut array: [u16; 31] = [0; 31];

    let mut i: u16 = 0;
    while i < 31 {
        array[i as usize] = 1 + i;
        i += 1;
    }

    array
};

pub const MESES: [u16; 12] = {
    let mut array: [u16; 12] = [0; 12];

    let mut i: u16 = 0;
    while i < 12 {
        array[i as usize] = 1 + i;
        i += 1;
    }

    array
};

const CNPJPESO1: [u32; 14] = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2, 0, 0];
const CNPJPESO2: [u32; 14] = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2, 0];
const NFEPESOS: [u32; 44] = [
    4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2, 9, 8, 7, 6, 5,
    4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2, 0,
];

/**
Formats the date in %d/%m/%Y format. Example:
```
    use unique::format_date;
    let dates = [
        " 1 / 1 / 2023 ",
        " 04/10/ 2018  17:04:11 ",
        "17/5/2014T12:34:56+09:30",
        "2014-5-17T12:34:56+09:30",
    ];
    let mut result = Vec::new();
    for date in dates {
        result.push(format_date(format_date(date)));
    }
    let valid = vec![
        "01/01/2023",
        "04/10/2018",
        "17/05/2014",
        "2014-5-17T12:34:56+09:30",
    ];
    assert_eq!(valid, result);
```
<https://rust-lang-nursery.github.io/rust-cookbook/datetime.html>

<https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
*/
pub fn format_date<T>(date: T) -> String
where
    T: Deref<Target = str> + std::fmt::Display,
{
    static DATE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\s*(\d{1,2})\s*/\s*(\d{1,2})\s*/\s*(\d{4})\s*T?\s*[\s\d+:]*$")
            .expect("DATE_REGEX: data inválida!")
    });

    match DATE_REGEX.captures(&date) {
        Some(caps) => {
            let dia = caps[1].parse::<u32>();
            let mes = caps[2].parse::<u32>();
            let ano = caps[3].parse::<i32>();
            match (dia, mes, ano) {
                (Ok(day), Ok(month), Ok(year)) => match NaiveDate::from_ymd_opt(year, month, day) {
                    Some(dt) => dt.format("%d/%m/%Y").to_string(),
                    None => date.to_string(),
                },
                _ => date.to_string(),
            }
        }
        None => date.to_string(),
    }
}

/**
Formats the date in %d/%m/%Y format. Example:
```
    use unique::format_date;
    let date = "1 / 1 / 2023";
    let result = format_date(date);
    assert_eq!(result, "01/01/2023");
```
<https://rust-lang-nursery.github.io/rust-cookbook/datetime.html>

<https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
*/
#[allow(dead_code)]
pub fn format_date_v2<T>(date: T) -> String
where
    T: Deref<Target = str> + std::fmt::Display,
{
    // %d Day number (01–31), zero-padded to 2 digits.
    // %m Month number (01–12), zero-padded to 2 digits.
    // %Y The full proleptic Gregorian year, zero-padded to 4 digits.
    match NaiveDate::parse_from_str(&date.replace(' ', ""), "%-d/%-m/%Y") {
        Ok(dt) => {
            //let dia = dt.day();
            //let mes = dt.month();
            //let ano = dt.year();
            //format!("{dia:02}/{mes:02}/{ano:04}")
            dt.format("%d/%m/%Y").to_string()
        }
        Err(_) => date.to_string(),
    }
}

// Entre o caracter ; (ponto e vírgula) e os 44 dígitos (\d{44}) pode ocorrer os caracteres ['\s] zero ou infinitas vezes
// teste; '''35120661156501000156550010000004551601580259;  --> teste;'35120661156501000156550010000004551601580259';
pub fn format_key<T>(chave: T) -> String
where
    T: Deref<Target = str>,
{
    let partes: Vec<&str> = chave
        .split('\'')
        .map(|c| c.trim())
        .filter(|&c| c.contains_only_digits())
        .filter(|&c| c.contains_num_digits(44))
        .collect();

    if partes.len() == 1 {
        let nfe = partes[0];
        let check: bool = check_nfe(nfe);
        if check {
            return ["'", nfe, "'"].concat();
        } else {
            return ["'", nfe, "' (dígito verificador Inválido!)"].concat();
        }
    }

    chave.to_string()
}

/// Formata strings numéricas para o padrão computational (f64)
/// com base no formato de entrada (Brasileiro ou Internacional).
pub fn format_number<T: AsRef<str>>(text: T, format: NumberFormat) -> String {
    let original = text.as_ref().trim();

    if original.is_empty() || original.chars().any(|c| c.is_alphabetic()) {
        return original.to_string();
    }

    // Define qual caractere indica que o número é decimal
    let decimal_sep = match format {
        NumberFormat::Brazilian => ',',
        NumberFormat::International => '.',
    };

    let mut cleaned = String::with_capacity(original.len());

    match format {
        NumberFormat::Brazilian => {
            for c in original.chars() {
                match c {
                    '.' => continue,          // Milhar BR: ignora
                    ',' => cleaned.push('.'), // Decimal BR: vira ponto
                    _ => cleaned.push(c),
                }
            }
        }
        NumberFormat::International => {
            for c in original.chars() {
                match c {
                    ',' => continue, // Milhar Intl: ignora
                    _ => cleaned.push(c),
                }
            }
        }
    }

    match cleaned.parse::<f64>() {
        Ok(num) => {
            let num = if num == 0.0 { 0.0 } else { num };
            let s = num.to_string();

            // REGRA: Só força ".0" se o separador DECIMAL original estava presente
            // e o resultado do to_string() não possui ponto decimal.
            if original.contains(decimal_sep) && !s.contains('.') {
                format!("{}.0", s)
            } else {
                s
            }
        }
        Err(_) => original.to_string(),
    }
}

/*
Chat GPT
Em liguagem de programação Rust,
dada uma linha com um texto qualquer,
escreva uma função que divide esta linha tal que
o parametro de divisão sejam números ou dígitos.

Reescreva a função acima de modo que
os dígitos vizinhos e os textos vizinhos devem permanecer juntos,
ou seja, dígitos vizinhos não podem ser divididos e também
os textos vizinhos não podem ser divididos.

Reescreva a função split_line_on_numbers adicionando comentários explicativos.
*/

/// Divide uma linha em partes, agrupando sequências de dígitos e sequências de não-dígitos.
///
/// Útil para processar textos que contêm números de documentos ou chaves misturados.
pub fn split_line_on_numbers(line: &str) -> Vec<String> {
    if line.is_empty() {
        return vec![];
    }

    let mut parts = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(c) = chars.next() {
        current.push(c);
        let next_is_digit = chars.peek().is_some_and(|n| n.is_ascii_digit());

        // Se a natureza do próximo caractere mudar (dígito vs não-dígito), fecha o grupo
        if chars.peek().is_some() && c.is_ascii_digit() != next_is_digit {
            parts.push(std::mem::take(&mut current));
        }
    }
    parts.push(current);
    parts
}

/// Valida se um CNPJ é matematicamente válido através dos dígitos verificadores.
pub fn check_cnpj(cnpj: &str) -> bool {
    let nums = cnpj.to_digits();
    if nums.len() != 14 {
        return false;
    }

    let calc_dv = |pesos: &[u32]| -> u32 {
        let soma: u32 = pesos.iter().zip(&nums).map(|(p, n)| p * n).sum();
        let resto = soma % 11;
        obter_dig_verificador(resto)
    };

    nums[12] == calc_dv(&CNPJPESO1) && nums[13] == calc_dv(&CNPJPESO2)
}

fn obter_dig_verificador(resto: u32) -> u32 {
    if resto < 2 {
        0
    } else {
        11 - resto
    }
}

/// element-wise multiplication for vecs
/// https://stackoverflow.com/questions/54603226/how-can-i-improve-the-performance-of-element-wise-multiplication-in-rust
fn vec_mul<T>(v1: &[T], v2: &[T]) -> T
where
    T: std::ops::Mul<Output = T> + std::iter::Sum + Copy,
{
    if v1.len() != v2.len() {
        panic!("Cannot multiply vectors of different lengths!")
    }

    v1.iter().zip(v2).map(|(&a, &b)| a * b).sum()
}

pub fn check_nfe(nfe: &str) -> bool {
    /*
    // Já filtrado anteriormente
    // nfe contém 44 dígitos numéricos
    if !nfe.contains_only_digit() {
        return false;
    }
    */

    let nums: Vec<u32> = nfe.to_digits();
    let resto: u32 = vec_mul(&NFEPESOS, &nums) % 11;
    let digito_verificador: u32 = obter_dig_verificador(resto);

    let cnpj = &nfe[6..20];
    let cnpj_valido = check_cnpj(cnpj);

    digito_verificador == nums[43] && cnpj_valido
}

#[cfg(test)]
mod functions {
    use super::*;
    use std::error::Error;

    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output

    // echo -n test | sha256sum
    // 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08

    pub const TEXT: &str = "test";

    #[test]
    fn multiplicacao_de_vetores() {
        // cargo test -- --show-output multiplicacao_de_vetores

        let a = [2.0, 3.0, 0.0, 1.0];
        let b = [0.0, 1.0, 7.2, 4.0];
        let res = vec_mul(&a, &b);

        println!("a = {a:?}");
        println!("b = {b:?}");
        println!("res = {res}");
        assert_eq!(res, 7.0);
    }

    #[test]
    fn check_nfe_e_cnpj() {
        // cargo test -- --show-output check_nfe_e_cnpj

        let nfes = vec![
            "29211212345678000195550010000000111000474894",
            "29211212345678000195550010000000121000476851",
            "29211212345678000195550010000000081000471555",
            "29211212345678000195550010000000091000471650",
            "24211212345678000195550020000000311356845330",
            "12345612345678000195550020000000341773403453",
        ];

        let mut verificacoes = Vec::new();

        for nfe in nfes {
            let validade: bool = check_nfe(nfe);
            verificacoes.push(validade);
            println!("nfe: '{nfe}' ; código verificador nfe: {validade}");

            let cnpj = &nfe[6..20];
            let cnpj_valido = check_cnpj(cnpj);
            println!("cnpj: '{cnpj}' ; código verificador cnpj: {cnpj_valido}\n");
        }

        assert_eq!(verificacoes, vec![true; 6]);
    }

    #[test]
    fn dividir_linha() {
        // cargo test -- --show-output dividir_linha

        let linha = " Este é um 3333 tes5te com div777__87 45 3 são pelos 543 dígitos 00. ";
        let splitted: Vec<String> = split_line_on_numbers(linha);
        println!("linha: '{linha}'");
        println!("splitted: '{splitted:#?}'");

        assert_eq!(
            splitted,
            vec![
                " Este é um ",
                "3333",
                " tes",
                "5",
                "te com div",
                "777",
                "__",
                "87",
                " ",
                "45",
                " ",
                "3",
                " são pelos ",
                "543",
                " dígitos ",
                "00",
                ". ",
            ]
        );
    }

    #[test]
    fn formatar_data() {
        // cargo test -- --show-output formatar_data

        let mut datas_formatadas = Vec::new();

        let datas = vec![
            "1 /1/ 2013",
            "7/11/2021 ",
            "  25 / 6 /   2015  ",
            "28/3/2253",
            "29/2/1973",
            "28/7b /2253",
            "0/7/2023",
        ];

        for data in datas {
            let data_formatada = format_date(data);
            let dt = ["'", data, "'"].concat();
            println!("data: {dt:<22} --> '{data_formatada}'");
            datas_formatadas.push(data_formatada);
        }

        assert_eq!(datas_formatadas[0], "01/01/2013");
        assert_eq!(datas_formatadas[1], "07/11/2021");
        assert_eq!(datas_formatadas[2], "25/06/2015");
        assert_eq!(datas_formatadas[3], "28/03/2253");
        assert_eq!(datas_formatadas[4], "29/2/1973");
        assert_eq!(datas_formatadas[5], "28/7b /2253");
        assert_eq!(datas_formatadas[6], "0/7/2023",);
    }

    #[test]
    fn blake3_hash_sum() -> Result<(), Box<dyn Error>> {
        // cargo test -- --show-output blake3_hash_sum

        // https://stackoverflow.com/questions/41069865/how-to-create-an-in-memory-object-that-can-be-used-as-a-reader-writer-or-seek/
        // https://nicholasbishop.github.io/rust-conversions/#path

        let hash: String = blake3::hash(TEXT.as_bytes()).to_string();

        println!("blake3({TEXT}): {hash}");
        let expected_result = "4878ca0425c739fa427f7eda20fe845f6b2e46ba5fe2a14df5b1e32f50603215";
        assert_eq!(&hash, expected_result);

        Ok(())
    }

    #[test]
    fn sub_multiple_whitespaces() {
        // cargo test -- --show-output sub_multiple_whitespaces

        let strings: Vec<&str> = vec![
            "teste",
            " teste",
            "teste ",
            " teste ",
            "  teste",
            "teste  ",
            "  teste  ",
            "tes te",
            "tes  te",
            "tes   te",
            " tes te",
            "tes  te ",
            " tes  te ",
            "  tes te",
            "tes  te  ",
            "  tes  te  ",
        ];
        for string in strings {
            let s = ["'", string, "'"].concat();
            println!("{:13} --> '{}'", s, string.replace_multiple_whitespaces());
        }
        let s1 = "tes  te".replace_multiple_whitespaces();
        let s2 = " tes  te".replace_multiple_whitespaces();
        let s3 = "tes  te ".replace_multiple_whitespaces();
        let s4 = " tes  te ".replace_multiple_whitespaces();
        let s5 = "  tes  te".replace_multiple_whitespaces();
        let s6 = "tes  te  ".replace_multiple_whitespaces();
        let s7 = "  tes  te  ".replace_multiple_whitespaces();

        assert_eq!(s1, "tes te");
        assert_eq!(s2, " tes te");
        assert_eq!(s3, "tes te ");
        assert_eq!(s4, " tes te ");
        assert_eq!(s5, " tes te");
        assert_eq!(s6, "tes te ");
        assert_eq!(s7, " tes te ");
    }
}

/// cargo test -- --show-output test_format_number
#[cfg(test)]
mod test_format_number {
    use super::*;
    use crate::args::NumberFormat;

    #[test]
    fn test_brazilian_format() {
        let br = NumberFormat::Brazilian;

        // Decimais e Milhares
        assert_eq!(format_number("1.234,56", br), "1234.56");
        assert_eq!(format_number("1.234.567,89", br), "1234567.89");
        assert_eq!(format_number("-1.500,7530", br), "-1500.753");

        // Inteiros com ponto (milhar)
        assert_eq!(format_number("1.000", br), "1000");
        assert_eq!(format_number("10.000", br), "10000");

        // Decimais puros
        assert_eq!(format_number("0,0012", br), "0.0012");
        assert_eq!(format_number("1,0000", br), "1.0");
        assert_eq!(format_number("1,65", br), "1.65");
        assert_eq!(format_number("7,60", br), "7.6");
        assert_eq!(format_number("1,6500", br), "1.65");
        assert_eq!(format_number("7,6000", br), "7.6");
    }

    #[test]
    fn test_international_format() {
        let intl = NumberFormat::International;

        // Decimais e Milhares
        assert_eq!(format_number("1,234.56", intl), "1234.56");
        assert_eq!(format_number("1,234,567.89", intl), "1234567.89");
        assert_eq!(format_number("-1,500.75", intl), "-1500.75");

        // Inteiros com vírgula (milhar)
        assert_eq!(format_number("1,000", intl), "1000");

        // Decimais puros
        assert_eq!(format_number("0.0012", intl), "0.0012");
        assert_eq!(format_number("1.0000", intl), "1.0");
    }

    #[test]
    fn test_semantic_differences() {
        // Teste crítico: a mesma string interpretada de formas diferentes

        let input_dot = "1.000";
        // BR: Interpreta ponto como milhar -> 1000
        assert_eq!(format_number(input_dot, NumberFormat::Brazilian), "1000");
        // INTL: Interpreta ponto como decimal -> 1.0
        assert_eq!(format_number(input_dot, NumberFormat::International), "1.0");

        let input_comma = "1,000";
        // BR: Interpreta vírgula como decimal -> 1.0
        assert_eq!(format_number(input_comma, NumberFormat::Brazilian), "1.0");
        // INTL: Interpreta vírgula como milhar -> 1000
        assert_eq!(
            format_number(input_comma, NumberFormat::International),
            "1000"
        );
    }

    #[test]
    fn test_normalization_and_zeros() {
        let br = NumberFormat::Brazilian;
        let intl = NumberFormat::International;

        // Normalização de zero negativo
        assert_eq!(format_number("-0,00", br), "0.0");
        assert_eq!(format_number("-0.00", intl), "0.0");

        // Zero puro (mantém como inteiro se não houver separador)
        assert_eq!(format_number("0", br), "0");
        assert_eq!(format_number("-0", intl), "0");

        // Manutenção do .0 para representar float
        assert_eq!(format_number("1,0", br), "1.0");
        assert_eq!(format_number("100,000", br), "100.0");
    }

    #[test]
    fn test_edge_cases_and_invalid_inputs() {
        let br = NumberFormat::Brazilian;

        // Letras invalidam a formatação (retorna original)
        assert_eq!(format_number("1.23b,50", br), "1.23b,50");
        assert_eq!(format_number("123a", br), "123a");
        assert_eq!(format_number("123 4", br), "123 4");
        assert_eq!(format_number("123 .4", br), "123 .4");
        assert_eq!(format_number("123 ,4", br), "123 ,4");
        assert_eq!(format_number("172835, 172834", br), "172835, 172834");

        // Strings vazias ou apenas espaços
        assert_eq!(format_number("", br), "");
        assert_eq!(format_number("   ", br), "");

        // Símbolos solitários ou malformados
        assert_eq!(format_number("+", br), "+");
        assert_eq!(format_number("-", br), "-");
        assert_eq!(format_number(".,.", br), ".,.");

        // Parse falho (ex: dois sinais)
        assert_eq!(format_number("++100,00", br), "++100,00");
        assert_eq!(format_number("--100,00", br), "--100,00");
    }

    #[test]
    fn test_large_numbers() {
        let br = NumberFormat::Brazilian;
        let intl = NumberFormat::International;

        assert_eq!(format_number("1.000.000.000,00", br), "1000000000.0");
        assert_eq!(format_number("1,000,000,000.00", intl), "1000000000.0");
        assert_eq!(format_number("-9.999.999,99", br), "-9999999.99");
    }

    #[test]
    fn test_integers_stay_integers() {
        let br = NumberFormat::Brazilian;
        let intl = NumberFormat::International;

        // Milhar no padrão BR não deve gerar .0
        assert_eq!(format_number("1.000", br), "1000");
        assert_eq!(format_number("1.234.567", br), "1234567");

        // Milhar no padrão Intl não deve gerar .0
        assert_eq!(format_number("1,000", intl), "1000");
        assert_eq!(format_number("1,234,567", intl), "1234567");

        // Zero puro
        assert_eq!(format_number("0", br), "0");
        assert_eq!(format_number("-0", br), "0");
    }

    #[test]
    fn test_decimals_keep_float_hint() {
        let br = NumberFormat::Brazilian;
        let intl = NumberFormat::International;

        // Se o usuário digitou o separador decimal, mantemos o .0
        assert_eq!(format_number("1,0", br), "1.0");
        assert_eq!(format_number("1.0", intl), "1.0");

        // Decimais reais
        assert_eq!(format_number("1.234,50", br), "1234.5");
        assert_eq!(format_number("1,234.50", intl), "1234.5");
    }
}
