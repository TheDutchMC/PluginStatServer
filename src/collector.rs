//! This module will collect metrics every interval N.
//! This isn't done directly in the metrics endpoint, due to the time it takes to collect the metrics from MysQL

use crate::appdata::AppData;
use mysql::prelude::{Queryable, FromValue};
use mysql::Row;
use std::fmt::Display;
use std::collections::HashMap;

const INTERVAL_SECNONDS: u64 = 120;

#[macro_export]
macro_rules! skip_fail {
        ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                eprintln!("An error occurred while fetching stats from the dabtase: {:?}", e);
                continue;
            }
        }
    };
}

pub fn spawn_collector(data: AppData) {
    std::thread::spawn(move || {
        loop {
            let java_versions = skip_fail!(select_to_map::<u64>(&data, "java_version"));
            let mc_versions = skip_fail!(select_to_map::<f64>(&data, "mc_version"));
            let os = skip_fail!(select_to_map::<String>(&data, "os"));
            let timezone = skip_fail!(select_to_map::<String>(&data, "timezone"));
            let player_avg = skip_fail!(get_avg(&data, "player_count"));
            let mem_avg = skip_fail!(get_avg(&data, "mem_mb"));

            for (k, v) in java_versions {
                &data.prom.java_version.with_label_values(&[&k]).set(v as i64);
            }

            for (k, v) in mc_versions {
                &data.prom.mc_versions.with_label_values(&[&k]).set(v as i64);
            }

            for (k, v) in os {
                &data.prom.os.with_label_values(&[&k]).set(v as i64);
            }

            for (k, v) in timezone {
                &data.prom.timezone.with_label_values(&[&k]).set(v as i64);
            }

            data.prom.player_avg.set(player_avg);
            data.prom.mem_mb_avg.set(mem_avg);

            std::thread::sleep(std::time::Duration::from_secs(INTERVAL_SECNONDS));
        }
    });
}

fn get_avg(data: &AppData, field: &str) -> Result<f64, String> {
    let mut conn = match data.pool.get_conn() {
        Ok(c) => c,
        Err(e) => return Err(e.to_string())
    };

    let sql_value = match conn.query::<Row, &str>(&format!("SELECT {field} FROM stats", field = field)) {
        Ok(v) => v,
        Err(e) => return Err(e.to_string())
    };

    let mut total = 0f64;
    let inc = sql_value.len();
    for row in sql_value {
        let v = row.get::<f64, &str>(field).unwrap();
        total += v;
    }

    Ok(total / (inc as f64))
}

fn select_to_map<T: FromValue + Display>(data: &AppData, field: &str) -> Result<HashMap<String, usize>, String> {
    let mut conn = match data.pool.get_conn() {
        Ok(c) => c,
        Err(e) => return Err(e.to_string())
    };

    let rows = match conn.query::<Row, &str>(&format!("SELECT {field} FROM stats GROUP BY uuid ORDER BY timestamp DESC", field = field)) {
        Ok(r) => r,
        Err(e) => return Err(e.to_string())
    };

    let mut result: HashMap<String, usize> = HashMap::new();
    for r in rows {
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