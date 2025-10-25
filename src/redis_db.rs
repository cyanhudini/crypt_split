use dotenv::dotenv;
use redis::{self, Commands};
use std::env;

pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
   
    pub fn create_from_env() -> redis::RedisResult<Self> {
        dotenv().ok();
        let url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let client = redis::Client::open(url.as_str())?;

        let mut con = client.get_connection()?;

        Ok(RedisClient { client })
    }

}

