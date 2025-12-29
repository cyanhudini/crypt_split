use crate::split::{FileChunkMetaData, FileData};
use dotenv::dotenv;
use redis::{self, Client, RedisResult, TypedCommands};
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

    pub fn store_chunk_metadata(file_data: &FileData) -> RedisResult<Option<FileData>> {
        let key = format!("file:{}", )
    }

    pub fn retrieve_chunk_metadata()-> RedisResult<()> {}

    pub fn delete_file_chun_metadata() -> RedisResult<()> {}

    //TODO: file_hash sollte später der datei name sein, return type eventuell zu () bzw. kännte es zum logging benutzt werden
    pub fn set_hvalue(
        &mut self,
        file_hash: &str,
        chunk_info: &FileData,
    ) -> redis::RedisResult<usize> {
        // https://stackoverflow.com/questions/78003329/why-does-resultmap-err-behave-differently-when-using-or-return
        // https://blog.ssanj.net/posts/2024-01-24-working-with-rust-result-part-10.html
        // map_err da nicht möglihc SerdeError in RedisError umzuwandeln. daher muss das mapping selbst übnernommen werden
        let serialized = serde_json::to_string(&chunk_info.chunks).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Fehler beim Serialisieren (Konvertiert von Serde zu RedisError)",
                e.to_string(),
            ))
        })?;
        let field = String::from("AA");
        let res = self.connection.hset(file_hash, field, serialized);
        return res;
    }

    pub fn delete_hkey(&mut self, file_hash: &str) -> RedisResult<usize> {
        let res = self.connection.del(file_hash)?;
        Ok(res)
    }

    pub fn get_hvalues(&mut self, file_name: &str) -> RedisResult<()> {
        let res = self.connection.get(file_name);
        Ok(())
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
        let chunk = FileChunkMetaData {
            index: 2,
            cloud_path: Some(String::from("s3://bucket/aaa")),
            previous_chunk_hash: Some(String::from("abcd1234")),
        };

        let file_data = FileData {
            file_name: String::from("testfile"),
            chunks: vec![chunk.clone()],
            hash_first_block: None,
            nonce: String::from("nonce1234"),
        };

        //let key = "test:set_key_value:1";

        let mut client = RedisClient::create_from_env().expect("Eoor beim Client Erstellen");
        let s_c_1 = serde_json::to_string(&chunk);

        let res = client
            .set_hvalue("test:SSSS", &file_data)
            .expect("hset failed");

        assert!(res >= 0);
    }
}
