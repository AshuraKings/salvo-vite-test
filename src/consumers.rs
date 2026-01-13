pub async fn start_consumer() -> anyhow::Result<()> {
    crate::configs::kafka::kafka_consuming("test", |msg| async move {
        tracing::info!("Processing message: {}", msg);
        Ok(())
    })
    .await?;
    Ok(())
}
