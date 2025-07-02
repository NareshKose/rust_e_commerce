use dotenvy::dotenv;
mod db;
mod handlers;
mod models;
mod routes;
mod utils;
mod middleware;
mod email;

use actix_web::{App, HttpServer};
use std::env;
use routes::auth_routes::auth_routes ;
use routes::product_routes::product_routes;

use routes::order_routes::order_routes;
use crate::routes::report_routes::report_routes;

use clickhouse::Client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let pool = db::init_mysql_pool().await;


 let clickhouse_client = Client::default()
        .with_url("http://localhost:8123")
        .with_database("default");

    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

     
    tokio::spawn(async {
        if let Err(e) = utils::kafka_consumer::run_kafka_consumer().await {
            eprintln!("Kafka consumer error: {:?}", e);
        }
    });

    tokio::spawn(async {
    if let Err(e) = utils::kafka_stock_consumer::run_stock_alert_consumer().await {
        eprintln!("Kafka consumer error: {:?}", e);
    }
});
    
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(pool.clone()))
            .app_data(actix_web::web::Data::new(clickhouse_client.clone()))
            .configure(auth_routes)
            .configure(product_routes)
            .configure(order_routes) 
            .configure(report_routes)  
    })
    .bind(bind_address)?
    .run()
    .await
}
