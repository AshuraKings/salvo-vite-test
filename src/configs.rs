use std::sync::{Arc, LazyLock};

use clap::Parser;
use tokio::sync::Mutex;

pub mod db;
pub mod kafka;

pub fn load_env() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    Ok(())
}

#[derive(Clone, Debug, Parser)]
pub struct AppConfig {
    #[arg(long, env = "HTTP_SERVER_PORT", default_value = "8698")]
    pub http_server_port: u16,
    #[arg(
        long,
        env = "DB_URL",
        default_value = "postgres://postgres:password@localhost:5432/postgres"
    )]
    pub db_url: String,
    #[arg(long, env = "KAFKA_BROKERS", default_value = "localhost:9092")]
    pub kafka_brokers: String,
    #[arg(long, env = "KAFKA_GROUP_ID", default_value = "salvo-group")]
    pub kafka_group_id: String,
}

static BG_RUNNING: LazyLock<Arc<Mutex<bool>>> = LazyLock::new(|| Arc::new(Mutex::new(false)));

pub async fn set_bg_running(running: bool) {
    let mut lock = BG_RUNNING.lock().await;
    *lock = running;
}

pub async fn is_bg_running() -> bool {
    let lock = BG_RUNNING.lock().await;
    *lock
}

static APP_CONFIG: LazyLock<Arc<Mutex<Option<AppConfig>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

pub async fn app_config_load() -> AppConfig {
    let mut lock = APP_CONFIG.lock().await;
    if lock.is_none() {
        tracing::info!("Loading APP_CONFIG");
        *lock = Some(AppConfig::parse());
    }
    lock.clone().unwrap()
}

pub async fn app_config_deletion() -> anyhow::Result<()> {
    let mut lock = APP_CONFIG.lock().await;
    if lock.is_some() {
        tracing::info!("Forgetting APP_CONFIG");
        *lock = None;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_app_config_load() {
        let _ = super::load_env();
        let app = super::app_config_load().await;
        assert_eq!(8698, app.http_server_port);
        let res = super::app_config_deletion().await;
        assert!(res.is_ok());
    }

    #[test]
    fn test_load_env() {
        let res = super::load_env();
        assert!(res.is_ok());
    }
}
