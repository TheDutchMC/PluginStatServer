use actix_web::{get, web, HttpResponse};
use crate::AppData;
use prometheus::{TextEncoder, Encoder, Registry, Gauge, Opts, IntGaugeVec};
use log::warn;
use std::collections::HashMap;

#[get("/metrics")]
pub async fn metrics(data: web::Data<AppData>) -> HttpResponse {
    let mut metrics = match data.metrics.lock() {
        Ok(m) => m,
        Err(e) => {
            warn!("Failed to lock metrics vector: {:?}", e);
            return HttpResponse::InternalServerError().finish()
        }
    };

    let player_avg = Gauge::new("pluginstat_player_avg", "Average player count on all servers").unwrap();
    let mem_mb_avg = Gauge::new("pluginstat_mem_mb_avg", "Average memory consumption on all servers").unwrap();

    let opts = Opts::new("pluginstat_java_version", "Java version used on Minecraft servers");
    let java_version = IntGaugeVec::new(opts, &["version"]).unwrap();

    let opts = Opts::new("pluginstat_mc_versions", "Minecraft versions used on Minecraft servers");
    let mc_versions = IntGaugeVec::new(opts, &["version"]).unwrap();

    let opts = Opts::new("pluginstat_os", "OS' used for Minecraft servers");
    let os = IntGaugeVec::new(opts, &["version"]).unwrap();

    let opts = Opts::new("pluginstat_timezone", "used for Minecraft servers");
    let timezone = IntGaugeVec::new(opts, &["version"]).unwrap();

    let reg = Registry::new();
    reg.register(Box::new(player_avg.clone())).unwrap();
    reg.register(Box::new(mem_mb_avg.clone())).unwrap();
    reg.register(Box::new(java_version.clone())).unwrap();
    reg.register(Box::new(mc_versions.clone())).unwrap();
    reg.register(Box::new(os.clone())).unwrap();
    reg.register(Box::new(timezone.clone())).unwrap();

    let mut player_avg_ct = 0u64;
    let mut mem_mb_avg_ct = 0u64;
    let mut java_version_ct = HashMap::new();
    let mut mc_versions_ct = HashMap::new();
    let mut os_ct = HashMap::new();
    let mut tz_ct = HashMap::new();

    let len = if metrics.len() == 0 {
        1
    } else {
        metrics.len()
    };

    let _ = metrics.drain(..).map(|metric| {
        player_avg_ct += metric.player_count;
        mem_mb_avg_ct += metric.mem_mb;
        *java_version_ct.entry(metric.java_version.to_string()).or_insert(0) += 1;
        *mc_versions_ct.entry(format!("{:.2}", metric.mc_version)).or_insert(0) += 1;
        *os_ct.entry(metric.os).or_insert(0) += 1;
        *tz_ct.entry(metric.timezone).or_insert(0) += 1;
    });

    player_avg.set(player_avg_ct as f64 / len as f64);
    mem_mb_avg.set(mem_mb_avg_ct as f64 / len as f64);
    let _ = java_version_ct.iter()
        .map(|(k, v)| java_version.with_label_values(&[k]).set(*v as i64));

    let _ = mc_versions_ct.iter()
        .map(|(k, v)| mc_versions.with_label_values(&[k]).set(*v as i64));

    let _ = os_ct.iter()
        .map(|(k, v)| os.with_label_values(&[k]).set(*v as i64));

    let _ = tz_ct.iter()
        .map(|(k, v)| timezone.with_label_values(&[k]).set(*v as i64));

    let families = reg.gather();
    let mut buff = Vec::new();
    let encoder = TextEncoder::new();
    encoder.encode(&families, &mut buff).expect("Failed to encode Prometheus metrics");

    HttpResponse::Ok().body(String::from_utf8(buff).expect("Failed to encode Prometheus metrics as a UTF-8 String"))
}