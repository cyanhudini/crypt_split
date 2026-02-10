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
        let serialized = serde_json::to_string(&file_data.chunks).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Fehler beim Serialisieren (Konvertiert von Serde zu RedisError)",
                e.to_string(),
            ))
        })?;
        self.connection.hset_multiple(
            &key,
            &[
                ("origin_block_hash", origin_block),
                ("nonce", file_data.nonce.clone()),
                ("chunks_count", chunks_count),
                ("wrapped_dek",file_data.wrapped_dek.clone().unwrap_or_default()),
                ("aad", file_data.aad.clone()),
                ("chunks", serialized)
            ],
        )?;

        
        //self.connection.hset(&key, "chunks", serialized)?;
        Ok(())
    }
    //TODO: eine update funktion implementieren um mehr Kontrolle zu haben im Falle wo Date bereits existiert,
    //wenn User z.B. aus Versehen doppelt verschlüsselt, sodass diese nicht überschrieben wird
    //check ob prüfsumme der Datei schon existiert z.B. durch file:{file_name}/checksum im Falle wo Name gleich ist
    //pub fn update_chunk_cloud_path() {}

    // https://redis.io/docs/latest/commands/HMGET/
    pub fn retrieve_chunk_metadata(&mut self, file_name: &str) -> RedisResult<Option<FileData>> {
        let key = format!("file:{}", file_name);

        let all_chunks_info = self.connection.hmget(
            &key,
            &[
                "origin_block_hash",
                "nonce",
                "chunks_count",
                "chunks",
                "wrapped_dek",
                "aad",
            ],
        )?;
        let ser_chunks = &all_chunks_info[3];

        let serialized: Vec<FileChunkMetaData> = serde_json::from_str(ser_chunks).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Fehler beim Serialisieren (Konvertiert von Serde zu RedisError)",
                e.to_string(),
            ))
        })?;

        let origin_block_hash = Some(all_chunks_info[0].clone());

        Ok(Some(FileData {
            file_name: file_name.to_string(),
            chunks: serialized,
            hash_first_block: origin_block_hash,
            nonce: all_chunks_info[1].clone(),
            wrapped_dek: Some(all_chunks_info[4].clone()),
            aad: all_chunks_info[5].clone(),
        }))
    }

    pub fn delete_file_chunk_metadata(&mut self, file_name: &str) -> RedisResult<usize> {
        let key = format!("file:{}", file_name);
        //self.connection.del(key)?;

        Ok(self.connection.del(key)?)
    }

    //https://redis.io/docs/latest/commands/ping/ für minimalen Health Check
    pub fn ping(&mut self) -> RedisResult<bool> {
        let pong: String = redis::cmd("PING").query(&mut self.connection)?;
        Ok(pong == "PONG")
    }

    /// Listet alle gespeicherten Dateien in der Redis-Datenbank auf
    /// Gibt eine Liste der Dateinamen zurück (ohne das "file:" Präfix)
    pub fn list_all_files(&mut self) -> RedisResult<Vec<String>> {
        // KEYS file:* gibt alle Keys zurück, die mit "file:" beginnen
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg("file:*")
            .query(&mut self.connection)?;

        // Entferne das "file:" Präfix von jedem Key
        let file_names: Vec<String> = keys
            .into_iter()
            .map(|key| key.strip_prefix("file:").unwrap_or(&key).to_string())
            .collect();

        Ok(file_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    //TODO: füge Test hinzu um hset mit einer echten Test Datei zu überprüfen
    #[test]
    fn test_ping() {
        let mut client = RedisClient::create_from_env().expect("Error beim Erstellen des Clients");
        client.ping().unwrap_or(false);
    }
}
