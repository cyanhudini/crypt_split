use aes_siv::{
    aead::{Aead, KeyInit, OsRng},
    Aes256SivAead, Nonce,
};

use password_hash::rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

// TODO: füge anyhow hinzu für konkretere Fehler

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileChunkMetaData {
    // TODO: Index muss entfernt werden, Ordnugn wird impliziert
    pub index: usize,
    pub cloud_path: Option<String>,
    pub chunk_hash: String,       
    pub previous_chunk_hash: String, 
}

#[derive(Debug)]
pub struct FileData {
    pub file_name: String,
    pub chunks: Vec<FileChunkMetaData>,
    pub hash_first_block: Option<String>,
    pub nonce: String,
}

const CHUNK_SIZE: usize = 4096;

pub fn split_file<P: AsRef<Path>, Q: AsRef<Path>>(file_path: P, output_path: Q, key: &[u8; 64],) -> io::Result<(FileData, PathBuf)> {
    /* 
    /home/nils/Uni/BA/split_hash_crypt_distr/chunks/48372587ac04466dbb4a4e0578925c74
     */

    let binding = Uuid::new_v4().to_string();
    let pre_split = binding.split("-");
    let output_folder = pre_split.collect::<String>();

    let output_folder_path = output_path.as_ref().join(output_folder);
    fs::create_dir_all(&output_folder_path)?;
    let mut input = Vec::new();
    File::open(file_path.as_ref())?.read_to_end(&mut input)?;
    let file_name = file_path
        .as_ref()
        .file_name()
        .and_then(|os| os.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| String::from("unknown"));
    //TODO: Nonce pro Datei generieren
    let mut nonce_bytes = [0u8; 16];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    //TODO: zeroization hinzufügen -> https://crates.io/crates/zeroize
    let encrypted_all = encrypt_with_aes_siv(&input, nonce, key);

    let mut chunks: Vec<FileChunkMetaData> = Vec::new();
    let file_size = encrypted_all.len();
    let mut index = 0;
    let mut bytes_red = 0;
    let mut first_block_hash: Option<String> = None;
    let mut prev_chunk_hash: Option<String> = None;
    while bytes_red < file_size {
        let read_size = std::cmp::min(bytes_red+CHUNK_SIZE, file_size );
        let chunk_buffer = &encrypted_all[bytes_red..read_size];
        let chunk_hex = hex::encode(chunk_buffer);
        let chunk_hash = hash_encrypted_data(&chunk_hex);

        if index == 0 {
            first_block_hash = Some(chunk_hash.clone());
        }

        let chunk_name = if let Some(ref prev_hash) = prev_chunk_hash {
            format!("{}_{}", chunk_hash, prev_hash)
        } else {
            chunk_hash.clone()
        };

        let chunk_path = output_folder_path.join(chunk_name.clone());
        let mut chunk_file = File::create(&chunk_path)?;
        chunk_file.write_all(chunk_buffer)?;
         //buffer needs to be hashed
        //let mut hasher = Sha256::new();
        //hasher.update(&buffer);
        //https://stackoverflow.com/questions/68694399/most-idiomatic-way-to-read-a-range-of-bytes-from-a-file
        //file.read_exact(&mut buffer)?;
        //hasher.finalize();
        // fürs erste der name der Datei
        
        //let chunk_name = format!("chunk_{}", index);
        // TODO: Hash des vorigen Chunks an den aktuellen hängen

        chunks.push(FileChunkMetaData {
            index,
            //TODO: cloud_path muss noch gesetzt werden
            cloud_path: None,
            chunk_hash: chunk_hash.clone(),
            previous_chunk_hash: prev_chunk_hash.clone().unwrap_or_default(),
        });
        prev_chunk_hash = Some(chunk_hash);
        index += 1;
        bytes_red = read_size;
    }

    Ok((FileData {
        file_name,
        chunks,
        hash_first_block: first_block_hash,
        nonce: hex::encode(nonce),
    }, output_folder_path))
}
// TODO: key als Paramter hinzufügen, Schlüssel durch KDF erzeugt werden, beim Starten des Programmes muss Passwort eingegeben werden
fn encrypt_with_aes_siv(plain_data: &Vec<u8>, nonce: &Nonce, key: &[u8; 64]) -> Vec<u8> {
    //let key = Aes256SivAead::generate_key(&mut OsRng);
    let cipher = Aes256SivAead::new_from_slice(key).expect("Falsche Länge des Keys");
    //let nonce = Nonce::from_slice(b"any unique nonce");
    let encrypted_data = cipher
        .encrypt(nonce, plain_data.as_ref())
        .expect("encryption failure!");
    //println!("Encrypted data: {:?}", encrypted_data);
    //println!("Encrypted data (hex): {}", hex::encode(&encrypted_data));
    encrypted_data
}

fn decrypt_with_aes_siv(encrypted_data: &[u8],nonce: &Nonce,key: &[u8; 64])-> Result<Vec<u8>, String>{
    let cipher = Aes256SivAead::new_from_slice(key).expect("Invalid key length");
    cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| format!("decryption failure: {}", e))
}

fn hash_encrypted_data(chunk_data: &String) -> String {

    let hash_result = Sha256::digest(chunk_data.as_bytes());
    let hash_string = format!("{:x}", hash_result);
    
    // fürs erste der name der Datei
    hash_string
}

//fürs erste nehmen wir an das wir nur einen Chunks Ordner haben
fn reconstruct_file<P: AsRef<Path>>(key : &[u8; 64], file_data: &FileData,chunks_folder: P,output_path: P){
    
    let encrypted_data: Vec<u8> = Vec::new();
    /*
    1. retrieve chunk metadata
    2. for(i in chunks)
            gehe all chunks durch und besorge Chunks von angegeben Ordner (später verschiedene Ordner)
            speichere diese in einen gemeinsamen Ordner
            lies chunks index 0
            dann index 1

     */     
}

fn check_integrity_block() {}

fn check_integrity_file(){}

#[cfg(test)]
mod tests {
    use super::*;
    /*
    TODO Tests:
    - 1 Block Split
    - >4Gb Split
    - korrektes Linking (ist der letzte Block wirklich der Vorgänger)
    - ob der erste Block nur ein Hash ist
    - ob Hash = Hash(chunk_data)
    - Integrität der ganzen Kette
    - decrypt (Schlüssel Management muss nochin implementiert werden)
    - 

     */
    #[test]
    fn test_split_file() {
       
        use std::path::PathBuf;
        /*if !Path::new("src/test.txt").exists() {
            let mut file = File::create("src/test.txt").unwrap();
            file.write_all(b"This is a test file with some data to split.")
                .unwrap();
        }*/
        let file_path = PathBuf::from("test/test_pdf.pdf");
        let output_path = PathBuf::from("test/output_chunks");

        //let result = split_file(file_path, output_path);

    }

    #[test]
    fn test_hash_chunk() {

        //hash_encrypted_data(&mut chunks);
        //TODO: Tests erweitern
    }
    #[test]
    fn test_encrypt_aes_siv() {
        let data = b"TO ENCRYPT";
        //TODO: Encrypt Test erweitern
    }
}
