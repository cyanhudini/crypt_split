use aes_gcm_siv::{
    aead::{Aead, KeyInit, OsRng, Payload},
    Aes256GcmSiv, Nonce,
};

use password_hash::rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::{
    fmt::format,
    fs::{self, File},
};
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
    pub wrapped_dek: Option<String>,
    pub aad: String,
}

const CHUNK_SIZE: usize = 4096;

pub fn split_file<P: AsRef<Path>>(
    file_path: P,
    output_path: P,
    key: &[u8; 32],
) -> io::Result<(FileData, PathBuf)> {
    /*
    /home/nils/Uni/BA/split_hash_crypt_distr/chunks/48372587ac04466dbb4a4e0578925c74
    */

    let binding = Uuid::new_v4().to_string();
    let pre_split = binding.split("-");
    let output_folder = pre_split.collect::<String>();

    let output_folder_path = output_path.as_ref().join(&output_folder);
    fs::create_dir_all(&output_folder_path)?;
    let mut input = Vec::new();
    File::open(file_path.as_ref())?.read_to_end(&mut input)?;
    let file_name = file_path
        .as_ref()
        .file_name()
        .and_then(|os| os.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| String::from("unknown"));
    //TODO: Nonce pro Datei generieren, statt die gesamte datei zu verschlüsseln, erst chunken dann verschlüsseln
    let mut nonce_bytes = [0u8; 12];
    //TODO: überlegen ob Nonce zufällig oder überhaupt benötigt wird
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    //TODO: zeroization hinzufügen -> https://crates.io/crates/zeroize
    // Build per-file AAD (authenticated but not encrypted). Store as hex in metadata.
    let aad_bytes = format!("{}:{}", output_folder, file_name).into_bytes();
    let encrypted_all = encrypt_with_aes_gcm_siv(&input, nonce, key, &aad_bytes);

    let mut chunks: Vec<FileChunkMetaData> = Vec::new();
    let file_size = encrypted_all.len();
    let mut index = 0;
    let mut bytes_red = 0;
    let mut first_block_hash: Option<String> = None;
    // Zufälligen Seed für den ersten Block generieren, damit er nicht erkennbar ist
    let mut seed_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut seed_bytes);
    let initial_seed = hex::encode(seed_bytes);
    let mut prev_chunk_hash: Option<String> = Some(initial_seed);
    while bytes_red < file_size {
        let read_size = std::cmp::min(bytes_red + CHUNK_SIZE, file_size);
        let chunk_buffer = &encrypted_all[bytes_red..read_size];
        let chunk_hex = hex::encode(chunk_buffer);
        let chunk_hash = hash_encrypted_data(&chunk_hex);

        if index == 0 {
            first_block_hash = Some(chunk_hash.clone());
        }

        // Alle Chunks haben jetzt das Format hash_prev-hash (inkl. erster Block mit Seed)
        let chunk_name = format!("{}_{}", chunk_hash, prev_chunk_hash.as_ref().unwrap());

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
            previous_chunk_hash: prev_chunk_hash.clone().unwrap(),
        });
        prev_chunk_hash = Some(chunk_hash);
        index += 1;
        bytes_red = read_size;
    }

    Ok((
        FileData {
            file_name,
            chunks,
            hash_first_block: first_block_hash,
            nonce: hex::encode(nonce),
            wrapped_dek: None,
            aad: hex::encode(&aad_bytes),
        },
        output_folder_path,
    ))
}
// TODO: key als Paramter hinzufügen, Schlüssel durch KDF erzeugt werden, beim Starten des Programmes muss Passwort eingegeben werden
fn encrypt_with_aes_gcm_siv(
    plain_data: &[u8],
    nonce: &Nonce,
    key: &[u8; 32],
    aad: &[u8],
) -> Vec<u8> {
    let cipher = Aes256GcmSiv::new_from_slice(key).expect("Falsche Länge des Keys");
    let payload = Payload {
        msg: plain_data,
        aad,
    };
    let encrypted_data = cipher.encrypt(nonce, payload).expect("encryption failure!");
    encrypted_data
}

fn decrypt_with_aes_gcm_siv(
    encrypted_data: &[u8],
    nonce: &Nonce,
    key: &[u8; 32],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher = Aes256GcmSiv::new_from_slice(key).expect("Invalid key length");
    let payload = Payload {
        msg: encrypted_data,
        aad,
    };
    cipher
        .decrypt(nonce, payload)
        .map_err(|e| format!("decryption failure: {}", e))
}

fn hash_encrypted_data(chunk_data: &String) -> String {
    let hash_result = Sha256::digest(chunk_data.as_bytes());
    let hash_string = format!("{:x}", hash_result);

    // fürs erste der name der Datei
    hash_string
}

pub fn reconstruct_file<P: AsRef<Path>, Q: AsRef<Path>>(
    key: &[u8; 32],
    file_data: &FileData,
    chunks_folder: P,
    output_path: Q,
) -> io::Result<PathBuf> {
    let mut encrypted_data: Vec<u8> = Vec::new();

    for chunk_meta in &file_data.chunks {
        // Alle Chunks haben jetzt das Format hash_prev-hash (inkl. erster Block mit Seed)
        let chunk_name = format!(
            "{}_{}",
            chunk_meta.chunk_hash, chunk_meta.previous_chunk_hash
        );

        let chunk_path = chunks_folder.as_ref().join(&chunk_name);
        let mut chunk_data = Vec::new();
        File::open(&chunk_path)?.read_to_end(&mut chunk_data)?;
        encrypted_data.extend(chunk_data);
    }

    let nonce_bytes =
        hex::decode(&file_data.nonce).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let aad_bytes =
        hex::decode(&file_data.aad).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let decrypted = decrypt_with_aes_gcm_siv(&encrypted_data, nonce, key, &aad_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let output_file_path = output_path.as_ref().join(&file_data.file_name);
    let mut output_file = File::create(&output_file_path)?;
    output_file.write_all(&decrypted)?;

    Ok(output_file_path)
}

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
    fn test_encrypt_aes_siv() {
        let data = b"TO ENCRYPT";
        //TODO: Encrypt Test erweitern
    }
}
