use actix_web::{HttpResponse, web, post};
use serde::Serialize;
use crate::appdata::AppData;
use rand::Rng;
use crate::common::Statistics;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    status:         u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    new_uuid:       Option<String>
}

#[post("/report")]
pub async fn report_stat(data: web::Data<AppData>, body: web::Json<Statistics>) -> HttpResponse {
    if body.uuid.is_empty() {
        let new_uuid: String = rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(64).map(char::from).collect();
        return HttpResponse::Ok().body(serde_json::to_string(&Response { status: 409, new_uuid: Some(new_uuid) }).unwrap());
    }

    match &data.tx.send(body.clone()) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to send Statistics over Channel: {:?}", e);
        }
    }

    HttpResponse::Ok().body(serde_json::to_string(&Response { status: 200, new_uuid: None }).unwrap())

}