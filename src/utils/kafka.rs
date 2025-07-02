use rdkafka::config::ClientConfig;

use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;


pub async fn send_to_kafka(topic: &str, payload: String) -> Result<(), Box<dyn std::error::Error>> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()?;

if let Err((e, _)) = producer
    .send(
        FutureRecord::to(topic).payload(&payload).key("order"),
        Timeout::Never,
    )
    .await
{
    eprintln!("Failed to send Kafka message: {}", e); 
}
    Ok(())
}
