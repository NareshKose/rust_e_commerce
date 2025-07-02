use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::ClientConfig;
use clickhouse::Client;
use chrono::Utc;

use crate::models::order::OrderProduct; 

pub async fn run_kafka_consumer() -> Result<(), Box<dyn std::error::Error>> {
    let consumer: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", "localhost:9092")
    .set("group.id", "order-consumer-group")
    .set("auto.offset.reset", "earliest")
    .create()
    .expect("Failed to create Kafka consumer");


    consumer.subscribe(&["order-topic"])?;

    println!("Kafka consumer started...");

    let client = Client::default()
        .with_url("http://localhost:8123")
         .with_user("default")
        .with_password("")   
        .with_database("default");

    loop {
        match consumer.recv().await {
            Err(e) => eprintln!("Kafka error: {:?}", e),
            Ok(msg) => {
                if let Some(payload) = msg.payload_view::<str>().transpose()? {
                    let orders: Vec<OrderProduct> = serde_json::from_str(payload)?;
                    for item in orders {
let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
  

    let insert = client
        .query("INSERT INTO ordered_products (order_id, product_id, product_name, quantity, total_price, timestamp) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&item.order_id)
        .bind(&item.product_id)
        .bind(&item.product_name)
        .bind(item.quantity)
        .bind(item.total_price.to_string()) // ClickHouse expects string for Decimal
        .bind(now)
        .execute()
        .await;
                        if let Err(err) = insert {
                            eprintln!("ClickHouse insert error: {:?}", err);
                        } else {
                            println!("Inserted into ClickHouse: {}",item.product_name);
                        }
                    }
                }
            }
        }
    }
}
