use actix_web::{get, web, HttpResponse};
use crate::appdata::AppData;
use prometheus::{TextEncoder, Encoder};

#[get("/metrics")]
pub async fn get_metrics(data: web::Data<AppData>) -> HttpResponse {
    let families = data.prom.registry.gather();
    let mut buff = Vec::new();

    let encoder = TextEncoder::new();
    encoder.encode(&families, &mut buff).unwrap();

    HttpResponse::Ok().body(String::from_utf8(buff).unwrap())
}

