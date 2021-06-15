use crate::appdata::AppData;
use std::thread::spawn;
use crate::common::Statistics;
use mysql::prelude::Queryable;
use mysql::{Params, Value};
use rand::Rng;

lazy_static! {
    static ref COLS: Vec<&'static str> = vec!["id", "uuid", "timestamp", "player_count", "mem_mb", "mc_version", "os", "java_version", "timezone"];
}

pub fn spawn_queue(data: AppData, rx: crossbeam_channel::Receiver<Statistics>) {

    //Receiver
    spawn(move || {
        let mut queue: Vec<Statistics> = Vec::new();
        loop {
            let recv = match rx.recv() {
                Ok(recv) => recv,
                Err(e) => {
                    eprintln!("Failed to receive statistics: {:?}", e);
                    continue;
                }
            };

            queue.push(recv);

            if queue.len() < 25 {
                continue;
            }

            let working_queue: Vec<Statistics> = queue.drain(..25).collect();

            let mut stmt = format!("INSERT INTO stats ({}) VALUES", COLS.join(","));
            let row = format!("({}),",
                              COLS.iter()
                                  .map(|_| "?".to_string())
                                  .collect::<Vec<_>>()
                                  .join(",")
            );

            stmt.reserve(working_queue.len() * (COLS.len() * 2 + 2));
            for _ in 0..working_queue.len() {
                stmt.push_str(&row);
            }

            //REmove trailing comma
            stmt.pop();

            let timestamp = chrono::Utc::now().timestamp();
            let mut params = Vec::with_capacity(working_queue.len() * COLS.len());
            for stat in working_queue {
                let stat_id: String = rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(64).map(char::from).collect();
                let vals: Vec<Value> = vec![
                    Value::from(stat_id),
                    Value::from(&stat.uuid),
                    Value::from(&timestamp),
                    Value::from(&stat.player_count),
                    Value::from(&stat.mem_mb),
                    Value::from(&stat.mc_version),
                    Value::from(&stat.os),
                    Value::from(&stat.java_version),
                    Value::from(&stat.timezone),
                ];

                params.extend(vals);
            }

            let mut conn = match data.pool.get_conn() {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Failed to create mysql connection: {:?}", e);
                    sleep(5);
                    continue;
                }
            };

            let params = Params::Positional(params);
            match conn.exec::<usize, String, Params>(stmt, params) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to insert bulk into database: {:?}", e);
                }
            }
        }
    });
}

fn sleep(secs: u64) {
    std::thread::sleep(std::time::Duration::from_secs(secs))
}