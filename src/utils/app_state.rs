use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use std::time::Duration;
use crate::utils::constants;

pub struct AppState {
    pub db: DatabaseConnection
}

impl AppState {
    pub async fn new(database_url: &str) -> Self {
        // let database_url = (*constants::DATABASE_URL).clone();
        let mut opt: ConnectOptions = ConnectOptions::new(database_url);
        opt.max_connections(32)
            .min_connections(4)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true);
        let db: DatabaseConnection = Database::connect(opt).await.unwrap();

        Self { db }
    }
}