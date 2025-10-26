use crate::split::FileChunk;
use dotenv::dotenv;
use redis::{self, Client, TypedCommands, RedisResult};
use serde_json;
use std::env;



pub struct RedisClient {
    connection: redis::Connection,
}
// TODO: implementiere SET, GET, DELETE und Basic Auth, Connection sollte langlebig sein
impl RedisClient {
    pub fn create_from_env() -> RedisResult<Self> {
        dotenv().ok();
        let url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let client = Client::open(url)?;
        let conn = client.get_connection()?;
        Ok(Self { connection: conn })
    }
    //TODO: file_hash sollte später der datei name sein, return type eventuell zu () bzw. kännte es zum logging benutzt werden
    pub fn set_hvalue(
        &mut self,
        file_hash: &str,
        chunk_info: &FileChunk,
    ) -> redis::RedisResult<usize> {
        let serialized = serde_json::to_string(&chunk_info).map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "Fehler beim Serialisieren (Konvertiert von Serde zu RedisError)", e.to_string())))?;
        let field = chunk_info.index.to_string(); 
        let res = self.connection.hset(file_hash, field, serialized);
        return res
        
    }

    pub fn delete_hkey(&mut self, file_hash: &str) -> RedisResult<usize> {
        let res = self.connection.del(file_hash)?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;


    //TODO: füge Test hinzu um hset mit einer echten Test Datei zu überprüfen

    #[test]
    fn test_set_key_value() {

        let chunk = FileChunk {
            index: 2,
            nonce: String::from("nonce1234"),
            cloud_path: Some(String::from("s3://bucket/aaa")),
        };

        //let key = "test:set_key_value:1";

        // create client
        let mut client = RedisClient::create_from_env().expect("Eoor beim Client Erstellen");
        let s_c_1 = serde_json::to_string(&chunk);
        // set the value
        client.set_hvalue(&String::from("test:SSSS"), &chunk);

        // read raw JSON back directly from the connection and deserialize
        

        // cleanup
        //let _removed: usize = client.connection.del(key).expect("del failed");
    }
}
