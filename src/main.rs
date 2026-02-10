use crate::cloud::disitribute_file_chunks;
use crate::split::{reconstruct_file, split_file};
use clap::{Parser, Subcommand};
use dotenv;
use redis::{self};
use serde::Deserialize;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use zeroize::Zeroize;
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

    EncryptThenDistribute {
        #[arg(short, long)]
        input_file: String,
        #[arg(short, long)]
        file_name: String,
        #[arg(short, long, default_value = "./chunks")]
        output_path: String,
    },
    /*
         arg(short, long)]
        input_file: PathBuf,
        #[arg(short, long, default_value="./chunks")]
        output_path : PathBuf,

    }*/
    Reconstruct {
        #[arg(short, long)]
        file_name: String,
        #[arg(short, long, default_value = "./output")]
        output_path: PathBuf,
    },

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
    if Path::new(KEY_FILE_PATH).exists() {
        //wenn leer, fdragen ob neu initaliseren
        print!("Existiert bereits");
        return Ok(());
    }

    let password = authorize_with_password()?;
    println!("Passwort");
    let s = key_management::init_kek_salt(&password, KEY_FILE_PATH)?;
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
fn cli_encrypt_and_split<P: AsRef<Path>>(file_path: P,output_path: P,password: &str,) -> io::Result<PathBuf> {
    let mut unlocked_key = key_management::load_kek_salt(password, KEY_FILE_PATH)?;
    //data encryption key wird für jede Datei neu generiert
    let mut dek = key_management::generate_dek();

    let (mut split_file_data, chunks_output_path) = split_file(file_path, output_path, &dek)?;
    println!("Output Pfad der Chunks: {:?}", chunks_output_path);
    let wrapped_dek = key_management::wrap_dek_key(&unlocked_key, &dek)?;
    split_file_data.wrapped_dek = Some(hex::encode(&wrapped_dek));
    unlocked_key.zeroize();
    dek.zeroize();
    let mut redis_client = redis_db::RedisClient::create_from_env()
        .map_err(|_| io::Error::new(ErrorKind::Other, "Fehler beim Erstellen des RedisClients"))?;
    redis_client
        .store_chunk_metadata(&split_file_data)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(chunks_output_path)
}

//TODO: Verteilung muss erst implementiert werden
fn cli_encrypt_and_distribute(
    file_path: &str,
    file_name: &str,
    output_path: &str,
    password: &str,
) -> io::Result<()> {
    let chunks_path = cli_encrypt_and_split(file_path, output_path, password)?;
    let chunks_path_str = chunks_path.to_string_lossy();
    cli_distribute_file_chunks(&chunks_path_str, file_name)?;
    Ok(())
}

fn list_all_stored_files() -> io::Result<()> {
    let mut redis_client = redis_db::RedisClient::create_from_env()
        .map_err(|_| io::Error::new(ErrorKind::Other, "Fehler beim Erstellen des RedisClients"))?;

    let files = redis_client
        .list_all_files()
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Redis Fehler: {}", e)))?;

    if files.is_empty() {
        println!("Keine Dateien in der Datenbank gefunden.");
    } else {
        println!("Gespeicherte Dateien ({}):", files.len());
        for (_, file_name) in files.iter().enumerate() {
            println!("{}", file_name);
        }
    }

    Ok(())
}

fn cli_distribute_file_chunks(chunks_path: &str, file_name: &str) -> io::Result<()> {
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
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Datei nicht in der Datenbank gefunden",
            )
        })?;
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
    let config_path = dotenv::var("CONFIG_PATH").unwrap_or_else(|_| "local_cloud.json".to_string());

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
        .map_err(|e| {
            io::Error::new(
                ErrorKind::Other,
                format!("Fehler beim Aktualisieren der Metadaten in Redis: {}", e),
            )
        })?;

    Ok(())
}


fn cli_reconstruct(file_name: &str, output_path: &Path, password: &str) -> io::Result<PathBuf> {
    let mut redis_client = redis_db::RedisClient::create_from_env()
        .map_err(|_| io::Error::new(ErrorKind::Other, "Fehler beim Erstellen des RedisClients"))?;

    let file_data = redis_client
        .retrieve_chunk_metadata(file_name)
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("Redis Fehler: {}", e)))?
        .ok_or_else(|| {
            io::Error::new(ErrorKind::NotFound, "Datei nicht in der Datenbank gefunden")
        })?;

    let temp_chunks_folder = output_path.join(".tmp_chunks");
    cloud::collect_chunks_to_folder(&file_data.chunks, &temp_chunks_folder)?;
    // load_kek -> load_dek -> unwrap_dek_with_kek()
    let mut kek = key_management::load_kek_salt(password, KEY_FILE_PATH)?;
    //as_ref() ist die kostengünstigere variante zu From/Into
    let wrapped_dek_h = file_data
        .wrapped_dek
        .as_ref()
        .expect("Fehler beim Laden des Wrapped DEK");
    let wrapped_dek = hex::decode(wrapped_dek_h)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("Hex decode Fehler: {}", e)))?;

    let mut dek = key_management::unwrap_key(&kek, &wrapped_dek)?;
    kek.zeroize();
    fs::create_dir_all(output_path)?;

    let output_file = reconstruct_file(&dek, &file_data, &temp_chunks_folder, output_path)?;
    dek.zeroize();

    fs::remove_dir_all(&temp_chunks_folder)?;

    Ok(output_file)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CLI::parse();
    match cli.command {
        Commands::Reconstruct {file_name,output_path,} => {
            let password = authorize_with_password()?;
            let output_file = cli_reconstruct(&file_name, &output_path, &password)?;
            println!("Datei rekonstruiert: {:?}", output_file);
        }
        Commands::Delete => {}
        Commands::Distr {
            chunks_path,
            file_name,
        } => {
            cli_distribute_file_chunks(&chunks_path, &file_name)?;
            /*
            1 Pfad angeben -> 2 Verteilen
             */
        }
        Commands::Encrypt {input_file,output_path,} => {
            let password = authorize_with_password()?;
            let chunks_path = cli_encrypt_and_split(input_file, output_path, &password)?;
            println!("Chunks gespeichert in: {:?}", chunks_path);
        }
        Commands::EncryptThenDistribute {input_file,file_name,output_path,
        } => {
            /*  1 Benutzerinteraktion(passworteingabe) -> 2 authorize_with_password(password) -> 3 DeEncryp(split())
                -> 4 Integrity(checksum_file()) ->5 Metadatenverwaltung(store_chunk_metadata())  ->  6 Metadatenverwaltung(store_checksum)
            */
            let password = authorize_with_password()?;
            let input_file =
                cli_encrypt_and_distribute(&input_file, &file_name, &output_path, &password)?;
        }
        Commands::Init => {
            /*
               1 Benutzerinteraktion(passworteingabe) -> 2 KeyManagement(initialize_master_key()) -> 3 KeyManagement(store_key)
            */
            start_init_key()?;
        }
        Commands::List => {
            list_all_stored_files()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    //TODO: Benchmarking mit DIVAN hinzufügen https://nikolaivazquez.com/blog/divan/
}
