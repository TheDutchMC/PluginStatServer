use actix_web::{get, web, HttpResponse};
use crate::appdata::AppData;
use prometheus::{TextEncoder, Encoder};
use mysql::prelude::{Queryable, FromValue};
use mysql::Row;
use std::collections::HashMap;
use std::fmt::Display;

#[get("/metrics")]
pub async fn get_metrics(data: web::Data<AppData>) -> HttpResponse {
    let java_versions = select_to_map::<u64>(&data, "java_version");
    let mc_versions = select_to_map::<f64>(&data, "mc_version");
    let os = select_to_map::<String>(&data, "os");
    let timezone = select_to_map::<String>(&data, "timezone");

    let java_versions = match java_versions.await {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to query java versions: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    for (k, v) in java_versions {
        &data.prom.java_version.with_label_values(&[&k]).set(v as i64);
    }

    let mc_versions = match mc_versions.await {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to query mc versions: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    for (k, v) in mc_versions {
        &data.prom.mc_versions.with_label_values(&[&k]).set(v as i64);
    }

    let os = match os.await {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to query OS: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    for (k, v) in os {
        &data.prom.os.with_label_values(&[&k]).set(v as i64);
    }

    let timezone = match timezone.await {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to query timezones: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    for (k, v) in timezone {
        &data.prom.timezone.with_label_values(&[&k]).set(v as i64);
    }

    let families = data.prom.registry.gather();
    let mut buff = Vec::new();

    let encoder = TextEncoder::new();
    encoder.encode(&families, &mut buff).unwrap();

    HttpResponse::Ok().body(String::from_utf8(buff).unwrap())
}

async fn select_to_map<T: FromValue + Display>(data: &AppData, field: &str) -> Result<HashMap<String, usize>, String> {
    let mut conn = match data.pool.get_conn() {
        Ok(c) => c,
        Err(e) => return Err(e.to_string())
    };

    let sql_java_versions = match conn.query::<Row, &str>(&format!("SELECT {field} FROM stats GROUP BY uuid ORDER BY timestamp DESC", field = field)) {
        Ok(r) => r,
        Err(e) => return Err(e.to_string())
    };

    let mut result: HashMap<String, usize> = HashMap::new();
    for r in sql_java_versions {
        let v = r.get::<T, &str>(field).unwrap();
        let v_str = format!("{}", v);

        {
            let existing_val = result.get(&v_str);
            if existing_val.is_some() {
                let val = existing_val.unwrap().clone();

                result.insert(v_str, val + 1usize);
                continue;
            }
        }

        result.insert(v_str, 1usize);
    }

    Ok(result)
}