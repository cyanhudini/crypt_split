use crate::split::FileChunk;
use dotenv::dotenv;
use redis::{self, Client, TypedCommands};
use serde_json;
use std::env;

pub struct RedisClient {
    connection: redis::Connection,
}
// TODO: implementiere SET, GET, DELETE und Basic Auth, Connection sollte langlebig sein
impl RedisClient {
    pub fn create_from_env() -> redis::RedisResult<Self> {
        dotenv().ok();
        let url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let client = Client::open(url)?;
        let conn = client.get_connection()?;
        Ok(Self { connection: conn })
    }

    pub fn set_key_value(
        &mut self,
        file_hash: &str,
        chunk_info: &FileChunk,
    ) -> redis::RedisResult<()> {}
}

