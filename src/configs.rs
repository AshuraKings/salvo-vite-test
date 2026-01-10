use std::sync::{Arc, LazyLock};

use clap::Parser;
use tokio::sync::Mutex;

pub fn load_env() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    Ok(())
}

#[derive(Clone, Debug, Parser)]
pub struct AppConfig {
    #[arg(long, env = "HTTP_SERVER_PORT", default_value = "8698")]
    pub http_server_port: u16,
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
    async fn test_app_config_deleting() {
        let res = super::app_config_deletion().await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_app_config_load() {
        let _ = super::load_env();
        let app = super::app_config_load().await;
        assert_eq!(8698, app.http_server_port);
    }

    #[test]
    fn test_load_env() {
        let res = super::load_env();
        assert!(res.is_ok());
    }
}
