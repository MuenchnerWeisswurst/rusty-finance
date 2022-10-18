mod api;
mod csv_model;
mod database;

extern crate encoding_rs;
extern crate encoding_rs_io;
#[macro_use]
extern crate log;

use axum::routing::put;
use axum::{routing::get, Router};
use axum_extra::routing::SpaRouter;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use dotenvy::dotenv;
use std::env;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;

use api::{data, hello};

const DATABASE_URL: &str = "DATABASE_URL";

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    match env::var(DATABASE_URL) {
        Ok(_) => (),
        Err(_) => {
            error!("DATABASE_URL must be set");
            panic!("DATABASE_URL must be set");
        }
    }
    let app = Router::new()
        .route("/api/alive", get(hello))
        .route("/api/data", put(data))
        .merge(SpaRouter::new("/assets", "dist").index_file("index.html"))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));
    let listen = env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("PORT")
        .map(|s| s.parse::<u16>().ok())
        .ok()
        .flatten()
        .unwrap_or(8080u16);

    println!("{}, {}", listen, port);
    let sock_addr = SocketAddr::from((
        IpAddr::from_str(&listen).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        port,
    ));

    println!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");
}
