#[macro_use]
extern crate envconfig_derive;
extern crate envconfig;
#[macro_use]
extern crate slog;
extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate failure;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use actix::System;
use actix_web::{web, App, HttpServer};
use failure::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod data;
mod external;
mod handlers;
mod logging;

const SECRETS_FILE: &str = "./me.secret";

use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "API_KEY", default = "")]
    pub api_key: String,

    #[envconfig(from = "API_SECRET", default = "")]
    pub api_secret: String,
}

#[derive(Debug)]
pub struct AppState {
    jwt: String,
    log: slog::Logger,
}

fn get_credentials(config: &Config) -> Result<(String, String), Error> {
    if config.api_key != "" && config.api_secret != "" {
        return Ok((config.api_key.to_string(), config.api_secret.to_string()));
    }
    let file = File::open(SECRETS_FILE).expect("Could not open file");
    let buf = BufReader::new(file);
    let lines: Vec<String> = buf
        .lines()
        .take(2)
        .map(std::result::Result::unwrap_or_default)
        .collect();
    if lines[0].is_empty() || lines[1].is_empty() {
        return Err(format_err!(
            "The first line needs to be the apiKey, the second line the apiSecret"
        ));
    }
    Ok((lines[0].to_string(), lines[1].to_string()))
}

fn main() {
    let mut sys = System::new("analyzer");

    let log = logging::setup_logging();
    let config = match Config::init() {
        Ok(v) => v,
        Err(e) => panic!("Could not read config from environment: {}", e),
    };
    let (api_key, api_secret) = match get_credentials(&config) {
        Ok(v) => v,
        Err(e) => panic!("Could not get credentials: {}", e),
    };
    info!(log, "Logging In...");
    let sign_in_response = match sys.block_on(external::get_jwt(&api_key, &api_secret)) {
        Ok(v) => v,
        Err(e) => panic!("Could not get the JWT: {}", e),
    };
    let jwt = sign_in_response.token;
    let runner_log = log.clone();
    info!(log, "Server Started on localhost:8080");
    match HttpServer::new(move || {
        App::new()
            .data(AppState {
                jwt: jwt.to_string(),
                log: log.clone(),
            })
            .service(
                web::scope("/rest/v1").service(
                    web::scope("/activities")
                        .service(
                            web::resource("")
                                .route(web::get().to_async(handlers::get_activities))
                                .route(web::post().to_async(handlers::create_activity)),
                        )
                        .service(
                            web::resource("/{activity_id}")
                                .route(web::get().to_async(handlers::get_activity))
                                .route(web::delete().to_async(handlers::delete_activity))
                                .route(web::patch().to_async(handlers::edit_activity)),
                        ),
                ),
            )
            .service(web::resource("/health").route(web::get().to(handlers::health)))
    })
    .bind("0.0.0.0:8080")
    .unwrap()
    .run()
    {
        Ok(_) => info!(runner_log, "Server Stopped!"),
        Err(e) => error!(runner_log, "Error running the server: {}", e),
    };
}
