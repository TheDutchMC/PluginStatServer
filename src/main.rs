use actix_web::{HttpServer, App};
use actix_web::middleware::Logger;

mod appdata;
mod endpoints;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    println!("Starting TheDutchMC Minecraft Plugin Statistics Server.");
    let env = match appdata::Env::new() {
        Ok(env) => env,
        Err(err) => {
            eprintln!("Unable to start: {}", err);
            std::process::exit(1);
        }
    };

    let appdata = match appdata::AppData::new(&env) {
        Ok(appdata) => appdata,
        Err(err) => {
            eprintln!("Unable to start: {}", err);
            std::process::exit(1);
        }
    };

    match appdata.check_db(&env) {
        Ok(passed) => {
            if !passed {
                println!("Database is incomplete. Correcting.");
                if let Err(e) = appdata.init_db() {
                    eprintln!("Failed to start: {}", e);
                    std::process::exit(1);
                };
            }
        },
        Err(e) => {
            eprintln!("Unable to start: {}", e);
            std::process::exit(1);
        }
    }

    HttpServer::new(move || {
        let cors = actix_cors::Cors::permissive();

        App::new()
            .data(appdata.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(endpoints::report_stat::report_stat)
            .service(endpoints::metrics::get_metrics)

    }).bind("0.0.0.0:8080")?.run().await
}