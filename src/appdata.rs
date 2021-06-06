use prometheus::{Gauge, Registry, Opts, IntGaugeVec};
use mysql::prelude::Queryable;
use mysql::{Row, Params, params};
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppData {
    pub pool:   mysql::Pool,
    pub prom:   Prom
}

impl AppData {
    pub fn new(env: &Env) -> Result<Self, String> {
        let env = env.clone();

        let mysql_uri = format!("mysql://{username}:{password}@{host}/{database}",
            username =  env.mysql_username,
            password =  env.mysql_password,
            host =      env.mysql_host,
            database =  env.mysql_database
        );

        let pool = mysql::Pool::new(mysql_uri);
        if pool.is_err() {
            return Err(pool.err().unwrap().to_string());
        }

        Ok(Self {
            pool: pool.unwrap(),
            prom: Prom::new()
        })
    }

    pub fn check_db(&self, env: &Env) -> Result<bool, String> {
        let mut conn = match self.pool.get_conn() {
            Ok(c) => c,
            Err(e) => {
                return Err(e.to_string());
            }
        };

        let sql_fetch_tables = match conn.exec::<Row, &str, Params>("SELECT table_name FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = :table_schema", params! {
            "table_schema" => &env.mysql_database
        }) {
            Ok(r) => r,
            Err(e) => {
                return Err(e.to_string());
            }
        };

        let mut required_tables_map = HashMap::new();
        required_tables_map.insert("stats".to_string(), false);

        for row in sql_fetch_tables {
            let table_name = row.get::<String, &str>("table_name").unwrap();
            required_tables_map.insert(table_name.clone(), true);
        }

        let mut db_passed = true;
        for entry in required_tables_map.iter() {
            if *entry.1 == false {
                eprintln!("Missing table: '{}'", entry.0);
                db_passed = false;
            }
        }

        Ok(db_passed)
    }

    pub fn init_db(&self) -> Result<(), String> {
        let mut conn = match self.pool.get_conn() {
            Ok(c) => c,
            Err(e) => {
                return Err(e.to_string());
            }
        };

        match conn.query::<usize, &str>("CREATE TABLE `stats` (`uuid` varchar(64) NOT NULL, `timestamp` bigint(20) NOT NULL, `player_count` int(11) NOT NULL, `mem_mb` bigint(20) NOT NULL, `mc_version` varchar(15) NOT NULL, `os` varchar(255) NOT NULL, `java_version` int(11) NOT NULL, `timezone` varchar(255) NOT NULL, PRIMARY KEY (`uuid`)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4") {
            Ok(_) => {},
            Err(e) => {
                return Err(e.to_string())
            }
        };

        Ok(())
    }
}

pub struct Env {
    mysql_host:     String,
    mysql_database: String,
    mysql_username: String,
    mysql_password: String
}

impl Env {
    pub fn new() -> Result<Self, &'static str> {
        use std::env::var;

        let mysql_host = var("MYSQL_HOST");
        if mysql_host.is_err() {
            return Err("Required environmental variable 'MYSQL_HOST' isn't set.");
        }

        let mysql_database = var("MYSQL_DATABASE");
        if mysql_host.is_err() {
            return Err("Required environmental variable 'MYSQL_DATABASE' isn't set.");
        }

        let mysql_username = var("MYSQL_USERNAME");
        if mysql_host.is_err() {
            return Err("Required environmental variable 'MYSQL_USERNAME' isn't set.");
        }

        let mysql_password = var("MYSQL_PASSWORD");
        if mysql_host.is_err() {
            return Err("Required environmental variable 'MYSQL_PASSWORD' isn't set.");
        }

        Ok(Self {
            mysql_host: mysql_host.unwrap(),
            mysql_database: mysql_database.unwrap(),
            mysql_username: mysql_username.unwrap(),
            mysql_password: mysql_password.unwrap()
        })
    }
}

#[derive(Clone)]
pub struct Prom {
    pub registry:       Registry,
    pub player_avg:     Gauge,
    pub mem_mb_avg:     Gauge,
    pub java_version:   IntGaugeVec,
    pub mc_versions:    IntGaugeVec,
    pub os:             IntGaugeVec,
    pub timezone:       IntGaugeVec
}

impl Prom {
    pub fn new() -> Self {
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

        let registry = Registry::new();
        registry.register(Box::new(player_avg.clone())).unwrap();
        registry.register(Box::new(mem_mb_avg.clone())).unwrap();
        registry.register(Box::new(java_version.clone())).unwrap();
        registry.register(Box::new(mc_versions.clone())).unwrap();
        registry.register(Box::new(os.clone())).unwrap();
        registry.register(Box::new(timezone.clone())).unwrap();

        Self {
            registry,
            player_avg,
            mem_mb_avg,
            java_version,
            mc_versions,
            os,
            timezone
        }
    }
}