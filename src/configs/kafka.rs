use rdkafka::{
    ClientConfig, ClientContext, Message,
    admin::{AdminClient, AdminOptions, NewTopic},
    consumer::{Consumer, ConsumerContext},
    producer::FutureProducer,
};

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {}

type StringConsumer = rdkafka::consumer::StreamConsumer<CustomContext>;

async fn kafka_config() -> anyhow::Result<ClientConfig> {
    let config = crate::configs::app_config_load().await;
    let brokers = config.kafka_brokers.clone();
    let group_id = config.kafka_group_id.clone();
    tracing::debug!("kafka_brokers = {} and group_id = {}", brokers, group_id);
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", brokers.as_str())
        .set("group.id", group_id.as_str())
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .set_log_level(rdkafka::config::RDKafkaLogLevel::Debug);
    Ok(config)
}

pub async fn kafka_producer() -> anyhow::Result<FutureProducer> {
    let config = kafka_config().await?;
    let producer = config.create::<FutureProducer>()?;
    Ok(producer)
}

pub async fn kafka_consuming<F, Fut>(topic: &str, func: F) -> anyhow::Result<()>
where
    F: Fn(String) -> Fut + Send + 'static + Copy,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    tracing::info!("consuming kafka topic: {}", topic);
    let context = CustomContext;
    let config = kafka_config().await?;
    let admin: AdminClient<CustomContext> = config.create_with_context(context)?;
    let new_topic = NewTopic::new(topic, 1, rdkafka::admin::TopicReplication::Fixed(1));
    admin
        .create_topics(&[new_topic], &AdminOptions::new())
        .await?;
    let context = CustomContext;
    let consumer: StringConsumer = config.create_with_context(context)?;
    consumer.subscribe(&[topic])?;
    tokio::spawn(async move {
        while crate::configs::is_bg_running().await {
            match consumer.recv().await {
                Ok(m) => {
                    tracing::debug!("Received Kafka message for topic: {}", m.topic());
                    if let Err(e) =
                        consumer.commit_message(&m, rdkafka::consumer::CommitMode::Async)
                    {
                        tracing::error!("Failed to commit message: {:?}", e);
                    } else {
                        if let Some(key) = m.key() {
                            let s = String::from_utf8_lossy(key).to_string();
                            tracing::debug!("Message key: {}", s);
                        }
                        if let Some(payload) = m.payload() {
                            let s = String::from_utf8_lossy(payload).to_string();
                            if let Err(e) = func(s).await {
                                tracing::error!("Error processing message: {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => tracing::error!("Kafka error: {:?}", e),
            }
        }
        consumer.unsubscribe();
    });
    Ok(())
}
