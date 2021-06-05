use actix_web::{HttpResponse, web, post};
use serde::{Deserialize, Serialize};
use mysql::prelude::Queryable;
use mysql::{Row, Params, params};
use crate::appdata::AppData;
use rand::Rng;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    uuid:           String,
    player_count:   u64,
    mem_mb:         u64,
    mc_version:     f64,
    os:             String,
    java_version:   u32,
    timezone:       String
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    status:         u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    new_uuid:       Option<String>
}

#[post("/report")]
pub async fn report_stat(data: web::Data<AppData>, body: web::Json<Statistics>) -> HttpResponse {
    let mut conn = match data.pool.get_conn() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to create mysql connection: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let existing_uuid_check = match conn.query::<Row, &str>("SELECT uuid FROM stats") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to query for existing UUIDs in the database: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if body.uuid.is_empty() {
        let new_uuid: String = rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(64).map(char::from).collect();
        return HttpResponse::Ok().body(serde_json::to_string(&Response { status: 409, new_uuid: Some(new_uuid) }).unwrap());
    }

    let timestamp = chrono::Utc::now().timestamp();
    let stat_id: String = rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(64).map(char::from).collect();

    let sql_insert = conn.exec::<usize, &str, Params>("INSERT INTO stats (id, uuid, timestamp, player_count, mem_mb, mc_version, os, java_version, timezone)\
    VALUES (:id, :uuid, :timestamp, :player_count, :mem_mb, :mc_version, :os, :java_version, :timezone)", params! {
        "id" => stat_id,
        "uuid" => &body.uuid,
        "timestamp" => timestamp,
        "player_count" => &body.player_count,
        "mem_mb" => &body.mem_mb,
        "mc_version" => &body.mc_version,
        "os" => &body.os,
        "java_version" => &body.java_version,
        "timezone" => &body.timezone
    });

    if sql_insert.is_err() {
        eprintln!("Failed to insert stats into mysql database: {:?}", sql_insert.err().unwrap());
        HttpResponse::InternalServerError().finish();
    }

    let avg_players = &data.prom.player_avg.get();
    let new_avg = (avg_players + *&body.player_count as f64) / ((data.prom.player_inc + 1) as f64);
    &data.prom.player_avg.set(new_avg);

    let avg_mem = &data.prom.mem_mb_avg.get();
    let new_avg = (avg_mem + *&body.mem_mb as f64) / ((data.prom.mem_inc + 1) as f64);
    &data.prom.mem_mb_avg.set(new_avg);

    HttpResponse::Ok().body(serde_json::to_string(&Response { status: 200, new_uuid: None }).unwrap())

}