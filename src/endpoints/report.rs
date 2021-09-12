use actix_web::{web, post, HttpResponse};
use serde::Serialize;
use crate::{AppData, Metric};
use log::warn;

#[derive(Serialize)]
struct Response {
    status: u16
}

#[post("/report")]
pub async fn report(data: web::Data<AppData>, payload: web::Json<Metric>) -> HttpResponse {
    let mut lock = match data.metrics.lock() {
        Ok(l) => l,
        Err(e) => {
            warn!("Failed to lock Metrics vector: {:?}", e);
            return HttpResponse::Ok().json(&Response {
                status: 500
            })
        }
    };

    lock.push((*payload).clone());
    HttpResponse::Ok().json(&Response { status: 200 })
}