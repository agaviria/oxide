#![recursion_limit = "128"]
#[macro_use] extern crate diesel;
#[macro_use] extern crate failure;
#[macro_use(lazy_static)] extern crate lazy_static;
#[macro_use] extern crate typename_derive;
// #[macro_use] extern crate validator_derive;

#[macro_use]
#[cfg(feature = "sentry")]
mod sentry;
mod config;
mod controllers;
mod error;
mod errr;
mod exception;
mod models;
mod payload;
mod rate_limit;
mod storage;
mod schema;
mod utils;

use std::env;

use chrono::Utc;
use config::CONF;
use dotenv::dotenv;
use femme::pretty::Logger;
use log;
use terminator::Terminator;
use warp::{self, Filter, http::Method, path};

fn main() -> Result<(), Terminator> {
    let conf = &CONF;
    dotenv().ok();
    Logger::new().start(conf.log_level_filter)?;
    log::info!("log mechanism initialized...");

    let db_pool = utils::pg_pool();
    let db = utils::pg(db_pool);

    let rate_limiter = rate_limit::leaky_bucket();

    let bundle_oxide = rate_limiter
        .and(
            warp::path!("version")
            .map(|| payload::ResponseBuilder::ok()
                .body(env!("CARGO_PKG_VERSION")
                )
            )
            .or(path!("time")
                .map(|| payload::ResponseBuilder::ok()
                    .body(Utc::now().to_rfc3339())
                )
            )
            .unify()
            .or(path!("users")
                .and(controllers::user::router(db.clone().boxed()))
            )
            .unify(),
        )
        .and(warp::header("Accept"))
        .map(|resp: payload::Response, _accept: String| {
            let mut http_resp_builder = warp::http::response::Builder::new();
            http_resp_builder.status(resp.status_code());
            http_resp_builder.header("Content-Type", "application/json");

            for (header, value) in resp.headers() {
                http_resp_builder.header(header.as_bytes(), value.clone());
            }

            match resp.value() {
                Some(value) => http_resp_builder
                    .body(serde_json::to_string(value).unwrap())
                    .unwrap(),
                None => http_resp_builder.body("".to_owned()).unwrap()
            }
        })
    .recover(utils::handle_rejection)
        .with(warp::log("oxide::api"))
        .with(
            warp::cors()
            .allow_any_origin()
            .allow_methods(vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(vec!["Authorization", "Content-Type"]),
        );

    warp::serve(bundle_oxide)
        .run(
            // localhost
            ([127, 0, 0, 1], 8080)
        );

    Ok(())
}
