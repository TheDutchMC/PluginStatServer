use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use serde::Deserialize;
use actix_web::{web, HttpServer, App};
use actix_web::middleware::Logger;
use std::sync::Mutex;

mod endpoints;

#[derive(Debug)]
pub struct AppData {
    metrics: Mutex<Vec<Metric>>
}

impl AppData {
    fn new() -> Self {
        Self {
            metrics: Mutex::new(vec![])
        }
    }
}


#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub uuid:           String,
    pub player_count:   u64,
    pub mem_mb:         u64,
    pub mc_version:     f64,
    pub os:             String,
    pub java_version:   u32,
    pub timezone:       String
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_logger();
    info!("Starting PluginStatServer");

    let data = web::Data::new(AppData::new());
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .service(crate::endpoints::metrics::metrics)
            .service(crate::endpoints::report::report)
    }).bind("[::]:8080")?.run().await
}

/// Setup log4rs with a console appender
fn setup_logger() {
    let config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder().build())))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .expect("Failed to create log4rs configuration");
    log4rs::init_config(config).expect("Failed to initialize log4rs");
}
