use axum::response::IntoResponse;
use diesel::prelude::*;
use diesel::upsert::*;
use diesel::PgConnection;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
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
    for _ in 0..11 {
        let mut skipped = String::new();
        let _ = buf_reader.read_line(&mut skipped);
        trace!("skipping: {}", skipped);
    }
    let mut csv_reader = ReaderBuilder::new().delimiter(b';').from_reader(buf_reader);

    let iter = csv_reader.deserialize::<CsvRow>();
    let db_rows = iter
        .filter_map(|row| {
            if let Ok(ir) = row {
                Some(Model::from(ir))
            } else {
                // Logging
                warn!("Parsing error: {:?}", row);
                None
            }
        })
        .collect::<Vec<_>>();
    // add labels (Dumb ml) https://docs.rs/linfa/0.6.0/linfa/?search=

    if log_enabled!(log::Level::Trace) {
        for ele in &db_rows {
            trace!("Upsert: {:?}", ele);
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
        Err(e) => error!("Insert failed! {:?}", e),
    }

    StatusCode::NO_CONTENT
}
