use std::path::{Path, PathBuf};
use std::{fs, io};
use rand::Rng;
use crate::split::FileChunkMetaData;
use std::io::{Write, ErrorKind};


#[derive(serde::Deserialize)]
pub struct LocalCloudConfig {
    local_cloud_paths : Vec<String>,
}

pub fn delete_tmp_chunks(){}

pub fn choose_random_folder(configs: &LocalCloudConfig) -> io::Result<String>{
    
    let paths = &configs.local_cloud_paths;
    if paths.is_empty() {
        return Err(io::Error::new(ErrorKind::InvalidInput, "Keine lokalen Cloud Pfade in der Konfiguration gefunden"));
    }
    
    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..paths.len());
    Ok(paths[random_index].clone())
}

pub fn disitribute_file_chunks<P: AsRef<Path>>(config: &str, chunks_folder_path: P, chunk_metadata : &mut Vec<FileChunkMetaData>, file_name: &str) -> io::Result<()>{
    
    //
    /*
    COnfig ist in Form einer JSON datei
    Wähle einen Ordner
    Update chunk metadata mit dem ausgewähltem Ordnerpfad

    dann werden die chunks in die Ordner bewegt aus dem chunks_folder_path 

    TODO: redis update_file_metadata muss vorher implementiert sein
    deserialize json
    paths = confiFile::open(config_path)

    for chunk in chunkmetadata {
        target_folder = choose_random_folder(paths)

        chunk_metadata.cloud_path = Some(target folder)
    }

     */

    let content = fs::read_to_string(config)?;
    // print!("{}", content);
    let config: LocalCloudConfig = serde_json::from_str(&content)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
    print!("FFF");

    
    for chunk in chunk_metadata {
        let target_folder = choose_random_folder(&config)?;
        // Chunk-Dateinamen: hash_previoushash (oder nur hash für den ersten Chunk)
        let chunk_file_name = if chunk.previous_chunk_hash.is_empty() {
            chunk.chunk_hash.clone()
        } else {
            format!("{}_{}", chunk.chunk_hash, chunk.previous_chunk_hash)
        };

        let source_path = chunks_folder_path.as_ref().join(&chunk_file_name);
        let target_path = PathBuf::from(&target_folder).join(&chunk_file_name);
        fs::rename(&source_path, &target_path)?;
        // chunk file metadate borrow -> dann cloud_path updaten und wieder freigeben, dann über redis updaten
        chunk.cloud_path = Some(target_folder);
    }
    
    Ok(())
}

