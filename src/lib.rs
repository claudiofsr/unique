pub mod args;
pub use args::*;

use std::ops::Deref;
use chrono::NaiveDate;
use regex::Regex;
use once_cell::sync::Lazy;

pub use claudiofsr_lib::StrExtension;

pub type MyError = Box<dyn std::error::Error + Send + Sync>;
pub type MyResult<T> = Result<T, MyError>;

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
    4, 3, 2,
    9, 8, 7, 6, 5, 4, 3, 2,
    9, 8, 7, 6, 5, 4, 3, 2,
    9, 8, 7, 6, 5, 4, 3, 2,
    9, 8, 7, 6, 5, 4, 3, 2,
    9, 8, 7, 6, 5, 4, 3, 2, 0
];

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
pub fn format_date<T>(date: T) -> String
where
    T: Deref<Target=str> + std::fmt::Display,
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
        },
        Err(_) => date.to_string()
    }
}

#[allow(dead_code)]
fn format_date_v2<T>(date: T) -> String
where
    T: Deref<Target=str>,
{
    /// Example:
    ///
    /// <https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html>
    static DATE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{4})$")
            .expect("DATE_REGEX regex inválida!")
    });

    match DATE_REGEX.captures(&date.replace(' ', "")) {
        Some(caps) => {
            let dia = caps[1].parse::<u16>().ok();
            let mes = caps[2].parse::<u16>().ok();
            let ano = caps[3].parse::<u16>().ok();
            match (dia, mes, ano) {
                (Some(d), Some(m), Some(a)) => {
                    if DIAS.binary_search(&d).is_ok() && MESES.binary_search(&m).is_ok() && a >= 2005 {
                        format!("{d:02}/{m:02}/{a:04}")
                    } else {
                        date.to_string()
                    }
                },
                _ => date.to_string(),
            }
        },
        None => date.to_string()
    }
}

// Entre o caracter ; (ponto e vírgula) e os 44 dígitos (\d{44}) pode ocorrer os caracteres ['\s] zero ou infinitas vezes
// teste; '''35120661156501000156550010000004551601580259;  --> teste;'35120661156501000156550010000004551601580259';
pub fn format_key<T>(chave: T) -> String
where
    T: Deref<Target=str>,
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

// ;-34.542.675,01;  --> ;-34542675.01;
// https://stackoverflow.com/questions/66714719/how-can-i-check-if-a-str-consists-of-only-a-given-slice-of-chars
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=104a8361285d2cbc21764f1f333ea428
pub fn format_number<T>(text: T) -> String
    where
    T: Deref<Target=str>,
{

    let my_digits: [char; 14] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '-', '.', ','];
    //let my_digits: Vec<u8> = "0123456789+-.,".bytes().collect();

    if text.is_empty() || !text.chars().all(|c| my_digits.contains(&c)) {
        return text.to_string();
    }

    let groups: Vec<&str> = text.split(['.', ',']).collect();
    //println!("groups: '{groups:?}' ; size: {}", groups.len());

    for group in groups {
        if group.is_empty() {
            return text.to_string();
        }
    }

    let mut new_str: String = text.to_string();
    new_str.retain(|current_char| {
        current_char == '.' || current_char == ','
    });
    let char_vec: Vec<char> = new_str.chars().collect();
    let size = char_vec.len();
    //println!("new_str: '{new_str}' ; char_vec: {char_vec:?} ; size: {size}");

    if size >= 3 {
        let first_char = char_vec[0];
        for current_char in &char_vec[1 .. size-1] {
            //println!("first_char: '{first_char}' ; current_char: '{current_char}'");
            if *current_char != first_char {
                return text.to_string();
            }
        }
    }

    let check1 = size >= 2 && char_vec[0] == '.' && char_vec[size-1] == ',';
    let check2 = size >= 2 && char_vec[0] == ',' && char_vec[size-1] == '.';
    let check3 = size == 1 && char_vec[0] == ',';

    let replaced = if check1 {
        text
        .replace('.',"")
        .replace(',',".")
    } else if check2 {
        text
        .replace(',',"")
    } else if check3 {
        text
        .replace(',',".")
    } else {
        text.to_string()
    };

    //println!("text: '{text}' --> replaced: {replaced}");

    match replaced.parse::<f64>() {
        Ok(num) => {
            //println!("parsed");
            num.to_string()
        },
        Err(_) => text.to_string()
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

// Esta função divide uma linha de texto em strings, mantendo os dígitos e os textos vizinhos juntos.
pub fn split_line_on_numbers (line: &str) -> Vec<String> {
    // Vetor que armazena as partes da linha de texto dividida
    let mut parts: Vec<String> = Vec::new();
    // Variável que armazena a parte atual da linha de texto
    let mut current_part = String::new();
    // Variável que indica se o caractere anterior era um dígito
    let mut prev_char_was_digit = false;

    // Itera sobre os caracteres da linha de texto
    for c in line.chars() {
        // Se o caractere atual é um dígito
        if c.is_ascii_digit() {
            // Se o caractere anterior também era um dígito
            if prev_char_was_digit {
                // Adiciona o caractere atual à parte atual
                current_part.push(c);
            } else {
                // Se a parte atual não é vazia
                if !current_part.is_empty() {
                    // Adiciona a parte atual ao vetor de partes
                    parts.push(current_part);
                    // Limpa a parte atual
                    current_part = String::new();
                }
                // Adiciona o caractere atual à parte atual
                current_part.push(c);
            }
            // Atualiza a variável para indicar que o caractere anterior era um dígito
            prev_char_was_digit = true;
        // Se o caractere atual não é um dígito
        } else {
            // Se o caractere anterior era um dígito
            if prev_char_was_digit {
                // Adiciona a parte atual ao vetor de partes
                parts.push(current_part);
                // Limpa a parte atual
                current_part = String::new();
            }
            // Adiciona o caractere atual à parte atual
            current_part.push(c);
            // Atualiza a variável para indicar que o caractere anterior não era um dígito
            prev_char_was_digit = false;
        }
    }

    // Se a parte atual não é vazia
    if !current_part.is_empty() {
        // Adiciona a parte atual ao vetor de partes
        parts.push(current_part);
    }
    // Retorna o vetor de partes
    parts
}

pub fn check_cnpj(cnpj: &str) -> bool {
    /*
    // Já filtrado anteriormente
    // cnpj contém 14 dígitos numéricos
    if !cnpj.contains_only_digit() {
        return false;
    }
    */

    let nums: Vec<u32> = cnpj.to_digits();
    let resto1: u32 = vec_mul(&CNPJPESO1, &nums) % 11;
    let resto2: u32 = vec_mul(&CNPJPESO2, &nums) % 11;

    let digito_verificador1: u32 = obter_dig_verificador(resto1);
    let digito_verificador2: u32 = obter_dig_verificador(resto2);

    digito_verificador1 == nums[12] && digito_verificador2 == nums[13]
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
            " teste", "teste ", " teste ",
            "  teste", "teste  ", "  teste  ",
            "tes te", "tes  te", "tes   te",
            " tes te", "tes  te ", " tes  te ",
            "  tes te", "tes  te  ", "  tes  te  ",
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

    #[test]
    fn formatar_numeros() {
        // cargo test -- --show-output formatar_numeros

        let strings = vec![
            "123456789y.,",
            "+4,567.,89",
            "+4,567.,.89",
            "+4,567,.89",
            "+4,567.1,89",
            "+4,567.22,89",
            "+4,567.333,89",
            "+4,567.89",
            "-4,567.89",
            "4,567.89",
            "4.567,89",
            "4567,89",
            "4567.89",
            "+4567.89",
            "-4567.89",
            "1.234.567,89",
            "1.234.567.333,89",
            "1,234,567,333.89",
            "1.234.567.89",
            "1.234,567,89",
            "7,532106",
            "-1.234.567,89",
            "1.23b.567,89",
            "1234567,89",
            "1234567.89",
            "123456789",
            "",
            " ",
            "+",
            "-",
            "+-",
            "321",
            "34.542.675,01",
            "34,542,675.01",
        ];

        for string in strings {
            println!("string: '{string}'");
            let num = format_number(string);
            let s = ["'", string, "'"].concat();
            println!("{s:>20} --> {num}\n");
        }

        assert_eq!(format_number("4.567,89"), "4567.89");
        assert_eq!(format_number("4,567.89"), "4567.89");
        assert_eq!(format_number("-67,89"), "-67.89");
    }
}
