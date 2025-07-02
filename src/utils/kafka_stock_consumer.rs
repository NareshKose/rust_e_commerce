use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::ClientConfig;
use serde_json::Value;
use crate::email::notifier::send_stock_alert_email;

pub async fn run_stock_alert_consumer() -> Result<(), Box<dyn std::error::Error>> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", "stock-alert-consumer")
        .set("auto.offset.reset", "latest")
        .create()?;

    consumer.subscribe(&["out-of-stock-topic"])?;

    println!("Stock Alert Kafka consumer started...");

    loop {
        match consumer.recv().await {
            Err(e) => eprintln!("Kafka error: {:?}", e),
            Ok(msg) => {
                if let Some(payload) = msg.payload_view::<str>().transpose()? {
                    let product: Value = serde_json::from_str(payload)?;

                    let product_name = product["product_name"].as_str().unwrap_or("Unknown");
                    let product_id = product["product_id"].as_str().unwrap_or("Unknown");
                    let remaining_stock = product["remaining_stock"].as_str().unwrap_or("Unknown");
                    let message = product["message"].as_str().unwrap_or("Unknown");

let email_message = format!(" {message} , product id : {product_id}, product name: {product_name}, remaining_stock :{remaining_stock} ");


                    // println!("Out of stock: {}", product_name);

                    if let Err(e) = send_stock_alert_email(email_message).await {
                        eprintln!("failed to send alert email: {:?}", e);
                    }
                }
            }
        }
    }
}

