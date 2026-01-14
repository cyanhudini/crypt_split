use crate::{key_management::load_and_unlock_key, split::split_file};
use clap::{Parser, Subcommand};
use redis::{self};
use serde::Deserialize;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use zeroize::Zeroize;
use dotenv;
mod cloud;
mod key_management;
mod redis_db;
mod split;


const KEY_FILE_PATH: &str = ".key_file";

// CLI wird nach Clap/Parser Muster gemacht https://docs.rs/clap/latest/clap/_cookbook/git_derive/index.html
#[derive(Parser, Debug)]
#[command(name = "crypt_split")]
struct CLI {
    #[command(subcommand)]
    command: Commands,
}
#[derive(serde::Deserialize)]
struct LocalCloudConfig {
    local_cloud_paths: Vec<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,

    Encrypt {
        #[arg(short, long)]
        input_file: PathBuf,
        #[arg(short, long, default_value = "./chunks")]
        output_path: PathBuf,
    },

    Distr {
        #[arg(short, long)]
        chunks_path: String,
        #[arg(short, long)]
        file_name: String,
    },

    EncryptThenDistirbute, /*{

                               #[arg(short, long)]
                               input_file: PathBuf,
                               #[arg(short, long, default_value="./chunks")]
                               output_path : PathBuf,

                           }*/

    Reconstruct,

    List,

    Delete,
}

/* wenn passwort Date existiert, fragen ob überschreiben, mit Hinweis das Verlust jeglicher Daten droht wenn
   authorize_with_password() (TODO: noch umbennen)
   key_managment::initialize_master_key() in .key_file schreiben
   key XOR mit Hash(password)
*/
fn start_init_key() -> io::Result<()> {
    //let password = "12345";
    let password = authorize_with_password()?;
    let (key_xor, salt) = key_management::initialize_master_key(&password);
    key_management::store_key(salt, key_xor, KEY_FILE_PATH)?;
    Ok(())
}

/* 1. encrypt/decrypt() -> passwort eingeben -> load_key() (hash(password) XOR XOR(key))
  2. init_key() -> passwort eingeben
*/
fn authorize_with_password() -> io::Result<String> {
    print!("Passwort eingeben:");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;

    Ok(password)
}

// EIngangspunkt der CLI fürs Splitten/Verteilen
fn cli_encrypt_and_split<P: AsRef<Path>, Q: AsRef<Path>>(file_path: P,output_path: Q,password: &str,) -> io::Result<()> {
    //TODO: load_key() erst implementieren
    /*
    key= load_and_unlock_key()
    file_data = split_file(input_file, output, key)
    let redis_client = start_redis_client()

    redis_client.store_metadata(file_data)



    */
    let mut unlocked_key = load_and_unlock_key(KEY_FILE_PATH, password)?;
    let (split_file_data, chunks_output_path) = split_file(file_path, output_path, &unlocked_key)?;
    print!("Output Pfad der Chunks: {:?}", chunks_output_path);
    //sobald der entsperrte Schlüssel nicht mehr gebraucht ist -> zeroize, aus Arbeitsspeicher entfernen
    unlocked_key.zeroize();
    let mut redis_client = redis_db::RedisClient::create_from_env()
        .map_err(|_| io::Error::new(ErrorKind::Other, "Fehler beim Erstellen des RedisCLients"))?;
    redis_client
        .store_chunk_metadata(&split_file_data)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}

//TODO: Verteilung muss erst implementiert werden
fn encrypt_and_distribute() {}

fn list_all_stored_files() {}

fn distribute_file_chunks(chunks_path: &str, file_name: &str) -> io::Result<()> {
    //cloud::disitribute_file_chunks()#
    let mut redis_client = redis_db::RedisClient::create_from_env()
        .map_err(|e| io::Error::new(ErrorKind::Other, "Fehler beim Erstellen des RedisCLients"))?;
    let mut file_data_option = redis_client
        .retrieve_chunk_metadata(file_name)
        .map_err(|e| {
            io::Error::new(
                ErrorKind::Other,
                "Fehler beim Abrufen der Dateimetadaten aus Redis",
            )
        })?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Datei nicht in der Datenbank gefunden"))?;
    // read env variable for config path
    /*
    example:
    {
    "local_cloud_paths" : [
        "/home/nils/Uni/BA/split_hash_crypt_distr/test/cloud1",
        "/home/nils/Uni/BA/split_hash_crypt_distr/test/cloud2",
        "/home/nils/Uni/BA/split_hash_crypt_distr/test/cloud3"
    ]
    }
    de-serialize json file to Vec<String>
     */
    let config_path = dotenv::var("CONFIG_PATH")
        .unwrap_or_else(|_| "local_cloud.json".to_string());

    //TODO: wenn Datei bereits existiert in Db soll Fehler ausgegeben werden, da sonst beim Distributing die Datei nicht gefunden wird
    // oder überschreiben?
    cloud::disitribute_file_chunks(
        &config_path,
        chunks_path.to_string(),
        &mut file_data_option.chunks,
        &mut file_data_option.file_name,
    )?;
    //TODO: Nachdem verteilt, soll chunks Ordner gelöscht werden
    // Update der Chunk-Metadaten in Redis mit den neuen cloud_paths
    redis_client
        .store_chunk_metadata(&file_data_option)
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Fehler beim Aktualisieren der Metadaten in Redis: {}", e)))?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CLI::parse();
    match cli.command {
        Commands::Reconstruct => {
            /*
               1A Benutzerinteraktion(List)/1B DeEncry(reconstruct()) -> 2 Benutzerinteraktion(authorize_with_password())
               3 KeyManagement(xor_passworthash_key()) -> DeEncry(reconstruct_file())

            */
        }
        Commands::Delete => {}
        Commands::Distr {
            chunks_path,
            file_name,
        } => {
            distribute_file_chunks(&chunks_path, &file_name)?;
            /*
            1 Pfad angeben -> 2 Verteilen
             */
        }
        Commands::Encrypt {
            input_file,
            output_path,
        } => {
            /*  1 Benutzerinteraktion(passworteingabe) -> 2 authorize_with_password(password) -> 3 KeyManagement(xor_passworthash_key())
                -> 4 DeEncryp(split()) -> 4 Integrity(checksum_file())
            */
            let password = authorize_with_password()?;
            let input_file = cli_encrypt_and_split(input_file, output_path, &password);
        }
        Commands::EncryptThenDistirbute => {
            /*  1 Benutzerinteraktion(passworteingabe) -> 2 authorize_with_password(password) -> 3 DeEncryp(split())
                -> 4 Integrity(checksum_file()) ->5 Metadatenverwaltung(store_chunk_metadata())  ->  6 Metadatenverwaltung(store_checksum)
            */
        }
        Commands::Init => {
            /*
               1 Benutzerinteraktion(passworteingabe) -> 2 KeyManagement(initialize_master_key()) -> 3 KeyManagement(store_key)
            */
            start_init_key()?;
        }
        Commands::List => {}
    }

    Ok(())
}


#[cfg(test)]
mod test {
    //TODO: Benchmarking mit DIVAN hinzufügen https://nikolaivazquez.com/blog/divan/
}