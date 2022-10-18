use super::super::csv_model::CsvRow;
use super::schema::transactions;
use chrono::NaiveDate;
use diesel::prelude::*;
#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = transactions)]
pub struct Model {
    id: i64,
    reservation: NaiveDate,
    receiver: String,
    tags: Vec<Option<String>>,
    amount: f64,
    currency: String,
}

impl From<CsvRow> for Model {
    fn from(row: CsvRow) -> Self {
        let id = row.get_id();
        Model {
            id,
            reservation: row.reservation,
            receiver: row.receiver,
            tags: row.tags.iter().map(String::to_owned).map(Some).collect(),
            amount: row.amount,
            currency: row.currency,
        }
    }
}
