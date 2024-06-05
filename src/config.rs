use crate::Result;

use std::fmt::Debug;

use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::fs;
use serde::Deserialize;
use serde_json;

const MAX_CONNECTIONS: u32 = 10;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: String,
    pub user: String,
    pub database: String, 
    pub port: String,
    pub password: String,
    pub max_connections: Option<u32>,
}

impl Config {
    pub async fn new(file_path: &str) -> Result<Config>  {
        let contents = fs::read_to_string(file_path).await?;
        let config: Config = serde_json::from_str(contents.as_str())?;

        Ok(config)
    }

    pub async fn connect(&self) -> Result<PgPool> {
        let url = format!("postgres://{}:{}@{}:{}/{}", self.user, self.password, self.server, self.port, self.database);

        let pool = PgPoolOptions::new()
            .max_connections(self.max_connections.unwrap_or(MAX_CONNECTIONS))
            .connect(&url)
            .await?;

        Ok(pool)
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "config: server: {} user: {} database: {} port: {} password: ****", 
        self.server, self.user, self.database, self.port)
    }
 }