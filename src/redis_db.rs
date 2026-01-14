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

        //TODO: Error Handling, eine ping_funktion für Health Check
        /* if self.ping()? == PONG
            Ok(Self)
            else
            Error(Keine Verbindung {E})
         */
        Ok(Self { connection: conn })
    }

    pub fn store_chunk_metadata(&mut self, file_data: &FileData) -> RedisResult<()> {
        let key = format!("file:{}", file_data.file_name);

        let origin_block = file_data.hash_first_block.clone().unwrap_or("".to_string());
        let chunks_count = file_data.chunks.len().to_string();
        // zwei redis Operationen: erst generelle Info setzen und dann die serialisierten Chunks

        self.connection.hset_multiple(&key, &[
            ("origin_block_hash", origin_block),
            ("nonce", file_data.nonce.clone()),
            ("chunks_count", chunks_count),
        ])?;

        let serialized = serde_json::to_string(&file_data.chunks).map_err(|e| {
        redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "Fehler beim Serialisieren (Konvertiert von Serde zu RedisError)",
            e.to_string(),
            ))
        })?;
        self.connection.hset(&key, "chunks", serialized)?;
        Ok(())

    }
    //TODO: eine update funktion implementieren um mehr Kontrolle zu haben im Falle wo Date bereits existiert,
    //wenn User z.B. aus Versehen doppelt verschlüsselt, sodass diese nicht überschrieben wird
    //check ob prüfsumme der Datei schon existiert z.B. durch file:{file_name}/checksum im Falle wo Name gleich ist
    pub fn update_chunk_cloud_path(){}

    // https://redis.io/docs/latest/commands/HMGET/
    pub fn retrieve_chunk_metadata(&mut self, file_name: &str)-> RedisResult<Option<FileData>> {
        let key = format!("file:{}",file_name);

        let all_chunks_info = self.connection.hmget(
            &key,
            &["origin_block_hash", "nonce", "chunks_count", "chunks"]
        )?;
        let ser_chunks = &all_chunks_info[3];

        let serialized: Vec<FileChunkMetaData> = serde_json::from_str(ser_chunks).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Fehler beim Serialisieren (Konvertiert von Serde zu RedisError)",
                e.to_string(),
            ))
        })?;

        let origin_block_hash = Some(all_chunks_info[1].clone());


        Ok(Some(FileData {
                file_name: file_name.to_string(),
                chunks: serialized,
                hash_first_block: origin_block_hash,
                nonce: all_chunks_info[2].clone()
            }))
    }

    pub fn delete_file_chunk_metadata(&mut self, file_name: &str) -> RedisResult<usize> {
        let key = format!("file:{}",file_name);
        //self.connection.del(key)?;
        
        Ok(self.connection.del(key)?)
    }

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
    //https://redis.io/docs/latest/commands/ping/ für minimalen Health Check
    pub fn ping(&mut self) -> RedisResult<bool>{
        let pong : String = redis::cmd("PING").query(&mut self.connection)?;
        Ok(pong == "PONG")
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    //TODO: füge Test hinzu um hset mit einer echten Test Datei zu überprüfen
    #[test]
    fn test_ping(){
        let mut client = RedisClient::create_from_env().expect("Error beim Erstellen des Clients");
        client.ping().unwrap_or(false);
    }
}
