use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;
use std::env;

pub async fn init_mysql_pool() -> MySqlPool {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create MySQL pool")
}
