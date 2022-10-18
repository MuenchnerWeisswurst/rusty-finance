use chrono::NaiveDate;
use serde::Deserialize;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
#[derive(Deserialize, Debug)]
pub struct CsvRow {
    #[serde(with = "date_format", rename = "Buchung")]
    pub reservation: NaiveDate,
    #[serde(with = "date_format", rename = "Valuta")]
    pub value_date: NaiveDate,
    #[serde(rename = "Auftraggeber/Empfänger")]
    pub receiver: String,
    #[serde(rename = "Buchungstext")]
    pub text: String,
    #[serde(with = "tag_format", rename = "Notiz")]
    pub tags: Vec<String>,
    #[serde(rename = "Verwendungszweck")]
    pub purpose: String,
    #[serde(with = "float_format", rename = "Betrag")]
    pub amount: f64,
    #[serde(rename = "Währung")]
    pub currency: String,
}

impl Hash for CsvRow {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.reservation.hash(state);
        self.value_date.hash(state);
        self.receiver.hash(state);
        self.text.hash(state);
        // tags can be changed, hence they are not used for an unique identifier
        //self.tags.hash(state);
        self.purpose.hash(state);
        format!("{}", self.amount).hash(state);
        self.currency.hash(state);
    }
}

impl CsvRow {
    pub fn get_id(&self) -> i64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let result = hasher.finish();
        i64::from_be_bytes(result.to_be_bytes())
    }
}

mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};
    const FORMAT: &str = "%d.%m.%Y";

    pub fn _serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

mod float_format {
    use serde::{self, Deserialize, Deserializer, Serializer};
    pub fn _serialize<S>(number: f32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&number.to_string())
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.replace('.', "")
            .replace(',', ".")
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
mod tag_format {
    use serde::{self, Deserialize, Deserializer, Serializer};
    pub fn _serialize<S>(tags: Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&tags.join("#"))
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(s.split('#')
            .map(String::from)
            .map(|s| s.replace('\r', "").replace('\n', ""))
            .filter(|s| !s.is_empty())
            .collect())
    }
}
