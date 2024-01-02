use serde::{de::Error, Serialize, Deserialize, Deserializer};
use chrono::NaiveDate;

// Examples:
// https://gist.github.com/ripx80/33f80618bf13e3f4964b0d75c62bfd28
// https://brokenco.de/2020/08/03/serde-deserialize-with-string.html
// https://github.com/serde-rs/json/issues/329
// https://serde.rs/custom-date-format.html

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PerDcomp {
     pub perdcomp: String,
     pub cnpj_declarante: String,
     #[serde(deserialize_with = "string_as_f64")]
     pub valor_total_do_credito: f64,
     #[serde(deserialize_with = "string_as_f64")]
     pub valor_do_credito_na_data_de_transmissao: f64,
     #[serde(deserialize_with = "string_as_f64")]
     pub valor_do_per: f64,
     #[serde(with = "my_date_format")]
     pub data_da_transmissao: NaiveDate,
     pub nome_empresarial: String,
     pub ua_declarante: String,
     pub cnpj_detentor_do_credito: String,
     pub ua_detentor_do_credito: String,
     pub tipo_de_declaracao: String,
     pub num_processo_atribuido_ao_perdcomp: String,
     pub num_processo_administrativo_anterior: String,
     pub num_processo_habilitado: String,
     pub tipo_do_documento: String,
     pub tipo_do_credito: String,
     pub trimestre_de_apuracao: String,
     pub situacao: String,
     pub motivo: String,
     #[serde(with = "my_date_format")]
     pub data_da_situacao: NaiveDate,
     pub perfil_do_contribuinte: String,
     pub mum_perdcomp_retificado_ou_cancelado: String,
     pub versao_do_pgd: String,
     pub cnpj_sucessora: String,
     pub ua_sucessora: String,
     pub processo_mesmo_credito: String,
     #[serde(default)]
     #[serde(with = "option_date")]
     pub data_da_distribuicao: Option<NaiveDate>,
     pub perdcomp_apenas_numeros: String,
     pub cnpj: String,
     pub cpf_responsavel_pela_analise: String,
     #[serde(default)]
     #[serde(with = "option_date")]
     pub data_dcomp_ativa: Option<NaiveDate>,
     pub motivo_de_interesse_fiscal: String,
     pub codigo_do_credito_apurado: String,
     pub base_legal: String,
     pub num_rpf: String,
     pub processos_judiciais: String,  
}

pub fn string_as_f64_v2<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|string| {
            // 1.234.567,89 => 1234567.89
            let s = string
                .trim()
                .replace('.', "")
                .replace(',', ".");
            s.parse::<f64>()
                .map_err({
                    //eprintln!("f64 Error: {string} -> {s}");
                    Error::custom
                })
        })
}

pub fn string_as_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let string_a: String = Deserialize::deserialize(deserializer)?;

    // 1.234.567,89 => 1234567.89
    let string_b = string_a
        .trim()
        .replace('.', "")
        .replace(',', ".");

    let result_float = string_b.parse::<f64>();

    let float: f64 = result_float
        .map_err({
            //eprintln!("f64 Error: {string_a} -> {string_b}");
            Error::custom
        })?;

    Ok(float)
}

/*
pub fn string_as_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    const FORMAT: &str = "%-d/%-m/%Y %H:%M:%S";
    let string = String::deserialize(deserializer)?;
    let date = NaiveDate::parse_from_str(&string, FORMAT)
        .map_err(serde::de::Error::custom)?;
    Ok(date)
}
*/

// Font: https://serde.rs/custom-date-format.html
mod my_date_format {
    use chrono::NaiveDate;
    use serde::{
        self,
        de::Error,
        Serializer,
        Deserialize,
        Deserializer,
    };

    const FORMAT: &str = "%-d/%-m/%Y %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &NaiveDate,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&string)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let dt = NaiveDate::parse_from_str(&string, FORMAT)
            .map_err({
                //eprintln!("NaiveDate Error: {string}");
                Error::custom
            })?;
        Ok(dt)
    }
}

// Font: https://stackoverflow.com/questions/44301748/how-can-i-deserialize-an-optional-field-with-custom-functions-using-serde
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=d4e3ff1407b518c7848a4ef31b4cf05c
// https://github.com/serde-rs/serde/issues/1425
mod option_date {
    use chrono::NaiveDate;
    use serde::{
        self,
        de::Error,
        Serializer,
        Deserialize,
        Deserializer,
    };

    const FORMAT: &str = "%-d/%-m/%Y %H:%M:%S";

    pub fn serialize<S>(date: &Option<NaiveDate>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref d) = *date {
            return s.serialize_str(&d.format("%d/%m/%Y").to_string());
        }
        s.serialize_none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            return Ok(Some(
                NaiveDate::parse_from_str(&s, FORMAT)
                .map_err({
                    //eprintln!("Option<NaiveDate> Error: {s:?}");
                    Error::custom
                })?
            ));
        }

        Ok(None)
    }
}