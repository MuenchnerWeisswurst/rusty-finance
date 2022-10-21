use axum::response::IntoResponse;
use csv::Trim;
use diesel::prelude::*;
use diesel::upsert::*;
use diesel::PgConnection;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::io::{BufRead, BufReader};

use crate::csv_model::CsvRow;
use crate::database::model::Model;
use crate::database::schema::{
    transactions,
    transactions::{id, tags},
};

use crate::DATABASE_URL;
use axum::http::StatusCode;
use csv::{self, ReaderBuilder};

pub(crate) async fn hello() -> impl IntoResponse {
    "hello from server!"
}

pub(crate) async fn data(text: String) -> StatusCode {
    let database_url = env::var(DATABASE_URL).unwrap();
    let parsed = text.split(',').map(|s| s.parse::<u8>());
    let mut bytes = Vec::new();
    for byte in parsed {
        if let Ok(b) = byte {
            bytes.push(b);
        } else {
            return StatusCode::BAD_REQUEST;
        }
    }
    debug!("Received {:?} bytes", bytes.len());
    // parse
    let mut buf_reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1252))
            .build(bytes.as_slice()),
    );
    // make this configurable?
    for _ in 0..12 {
        let mut skipped = String::new();
        let _ = buf_reader.read_line(&mut skipped);
        trace!("skipping: {}", skipped);
    }
    let csv_reader = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .flexible(true)
        .trim(Trim::All)
        .from_reader(buf_reader);

    let parsed_db_rows = csv_reader.into_records()
        .map(|row| {
            if let Ok(ir) = row {
                let csv_row:Result<CsvRow, _> = ir.try_into();
                if let Err(e) = csv_row {
                    error!("{:?}", e);
                    return None;
                }
                Some(Model::from(csv_row.unwrap()))
            } else {
                // Logging
                warn!("Parsing error: {:?}", row);
                None
            }
        })
        .collect::<Vec<_>>();
    // add labels (Dumb ml) https://docs.rs/linfa/0.6.0/linfa/?search=
    if parsed_db_rows.iter().any(Option::is_none) {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let db_rows = parsed_db_rows
        .iter()
        .map(|o| o.as_ref().unwrap())
        .collect::<Vec<_>>();

    if log_enabled!(log::Level::Trace) {
        for ele in &db_rows {
            trace!("Upsert: {:?}", ele);
        }
        let data_len = db_rows.len();
        let set_id = db_rows.iter().map(|r| r.id).collect::<HashSet<_>>();
        if set_id.len() != data_len {
            trace!("{} duplicated keys", data_len - set_id.len());
            let mut map: HashMap<i64, Vec<&Model>> = HashMap::new();
            for e in &db_rows {
                map.entry(e.id).or_insert(Vec::new()).push(e);
            }
            let dups = map.values().filter(|v| v.len() > 1).collect::<Vec<_>>();
            trace!("Filtered {}", dups.len());
            for dup in dups {
                trace!("Duplicated rows:\n\t{:?}\n\t{:?}", dup[0], dup[1])
            }
        }
    }
    let conn = PgConnection::establish(&database_url);
    if let Err(e) = conn {
        error!("Error connecting to db: {:?}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    // write to db
    let result = diesel::insert_into(transactions::table)
        .values(db_rows)
        .on_conflict(id)
        .do_update()
        .set(tags.eq(excluded(tags)))
        .execute(&mut conn.unwrap());
    match result {
        Ok(n_bytes) => info!("Inserted/Updated {} row", n_bytes),
        Err(e) => {
            error!("Insert failed! {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    StatusCode::NO_CONTENT
}
