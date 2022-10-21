use anyhow::anyhow;
use chrono::NaiveDate;
use csv::StringRecord;
use phf::{phf_map, Map};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

static WHOLE_FIELD_TO_IDX: Map<&'static str, usize> = phf_map! {
    "RESERVATION" => 0,
    "VALUE_DATE" => 1,
    "RECEIVER" => 2,
    "TEXT" => 3,
    "TAGS" => 4,
    "PURPOSE" => 5,
    "BALANCE" => 6,
    "CURRENCY" => 7,
    "AMOUNT" => 8,
};
static PARTIAL_FIELD_TO_IDX: Map<&'static str, usize> = phf_map! {
    "RESERVATION" => 0,
    "VALUE_DATE" => 1,
    "RECEIVER" => 2,
    "TEXT" => 3,
    "TAGS" => usize::MAX,
    "PURPOSE" => 4,
    "BALANCE" => 5,
    "CURRENCY" => 6,
    "AMOUNT" => 7,
};

const FORMAT: &str = "%d.%m.%Y";
#[derive(Debug)]
pub struct CsvRow {
    pub reservation: NaiveDate,
    pub value_date: NaiveDate,
    pub receiver: String,
    pub text: String,
    pub tags: Vec<String>,
    pub purpose: String,
    pub balance: f64,
    pub currency: String,
    pub amount: f64,
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
        format!("{}", self.balance).hash(state);
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

impl TryFrom<StringRecord> for CsvRow {
    fn try_from(record: StringRecord) -> Result<Self, anyhow::Error> {
        let idx_map: &Map<&str, usize>;
        if record.len() == 10 {
            idx_map = &WHOLE_FIELD_TO_IDX;
        } else if record.len() == 9 {
            idx_map = &PARTIAL_FIELD_TO_IDX;
        } else {
            return Err(anyhow!(
                "Record has only {} fields but 9 or 10 are required! {:?}",
                record.len(),
                record
            ));
        }

        let res = record
            .get(*idx_map.get("RESERVATION").unwrap())
            .and_then(|d| NaiveDate::parse_from_str(d, FORMAT).ok());
        let value_date = record
            .get(*idx_map.get("VALUE_DATE").unwrap())
            .and_then(|d| NaiveDate::parse_from_str(d, FORMAT).ok());
        let rec = record
            .get(*idx_map.get("RECEIVER").unwrap())
            .map(String::from);
        let tags = record
            .get(*idx_map.get("TAGS").unwrap())
            .map(|s| {
                s.split('#')
                    .map(String::from)
                    .map(|s| s.replace('\r', "").replace('\n', ""))
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let balance = record
            .get(*idx_map.get("BALANCE").unwrap())
            .and_then(|s| s.replace('.', "").replace(',', ".").parse::<f64>().ok());
        let curr = record
            .get(*idx_map.get("CURRENCY").unwrap())
            .map(String::from);
        let amount = record
            .get(*idx_map.get("AMOUNT").unwrap())
            .and_then(|s| s.replace('.', "").replace(',', ".").parse::<f64>().ok());

        let text = record.get(*idx_map.get("TEXT").unwrap()).map(String::from);
        let purpose = record
            .get(*idx_map.get("PURPOSE").unwrap())
            .map(String::from);
        if res.is_none()
            || value_date.is_none()
            || rec.is_none()
            || text.is_none()
            || purpose.is_none()
            || balance.is_none()
            || amount.is_none()
            || curr.is_none()
        {
            return Err(anyhow!("Unable to parse {:?}", record));
        }
        Ok(CsvRow {
            reservation: res.unwrap(),
            value_date: value_date.unwrap(),
            receiver: rec.unwrap(),
            text: text.unwrap(),
            tags,
            purpose: purpose.unwrap(),
            balance: balance.unwrap(),
            amount: amount.unwrap(),
            currency: curr.unwrap(),
        })
    }

    type Error = anyhow::Error;
}
