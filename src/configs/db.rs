use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::sync::Mutex;

static DB_POOL: LazyLock<Arc<Mutex<Option<DatabaseConnection>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

pub async fn db_pool_load() -> anyhow::Result<DatabaseConnection> {
    let mut lock = DB_POOL.lock().await;
    if lock.is_none() {
        let app = crate::configs::app_config_load().await;
        let mut opt = ConnectOptions::new(app.db_url.as_str());
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true) // disable SQLx logging
            .sqlx_logging_level(log::LevelFilter::Info)
            .set_schema_search_path("public");
        let db = Database::connect(opt).await?;
        Migrator::up(&db, None).await?;
        *lock = Some(db);
    }
    Ok(lock.clone().unwrap())
}

pub async fn db_pool_deletion() -> anyhow::Result<()> {
    let mut lock = DB_POOL.lock().await;
    if let Some(db) = lock.take() {
        db.close().await?;
        *lock = None;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use migration::MigratorTrait;

    #[tokio::test]
    async fn test_db() {
        let _ = crate::configs::load_env();
        let res_db = super::db_pool_load().await;
        assert!(res_db.is_ok());
        let db = res_db.unwrap();
        assert!(db.ping().await.is_ok());
        let res_revert = migration::Migrator::down(&db, None).await;
        assert!(res_revert.is_ok());
        let res = super::db_pool_deletion().await;
        assert!(res.is_ok());
    }
}
