use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use aes_siv::{
    aead::{Aead, KeyInit, OsRng},
    Aes256SivAead, Nonce
};
use uuid::Uuid;

pub struct FileChunk {
    pub index: usize,
    pub file_path: PathBuf,
    pub size: usize,
    pub encrypted_data: String,
    pub nonce: String,
    pub cloud_path: Option<String>,
}

const CHUNK_SIZE: usize = 4096;

fn split_file<P: AsRef<Path>, Q: AsRef<Path>>(
    file_path: P,
    output_path: Q,
) -> io::Result<Vec<FileChunk>> {
    /*
    die gechnkten dateien sollen in einen ordner gespeichert werden, der name des ordners ist eine uuid
    der user sollte den pfad angeben wo der ordner erstellt werden soll
    */
    let output_folder = Uuid::new_v4().to_string();
    let output_folder_path = output_path.as_ref().join(output_folder);
    fs::create_dir_all(&output_folder_path)?;

    let mut file = File::open(file_path.as_ref())?;
    let mut chunks: Vec<FileChunk> = Vec::new();
    let file_size = file.metadata()?.len() as usize;
    let mut index = 0;
    let mut bytes_red = 0;
    //let output_folder = Uuid::new_v4().to_string();
    while bytes_red < file_size {
        let rest = file_size - bytes_red;
        let read_size = std::cmp::min(rest, CHUNK_SIZE);

        let mut buffer = vec![0u8; read_size];
        let (encrypted_data, nonce) = encrypt_with_aes_siv(&buffer);
        let chunk_name = hash_encrypted_data(&encrypted_data);
        //buffer needs to be hashed
        //let mut hasher = Sha256::new();
        //hasher.update(&buffer);
        file.read_exact(&mut buffer)?;
        //hasher.finalize();
        // fürs erste der name der Datei
        
        //let chunk_name = format!("chunk_{}", index);
        let chunk_path = output_folder_path.join(chunk_name);
        // erstellen der Datei passiert erst nach dem Hashen
        let mut chunk_file = File::create(&chunk_path)?;

        chunk_file.write_all(&buffer)?;
        // TODO: metadaten index, filepath, size usw. sind nachher wichtig für Tabelle
        chunks.push(FileChunk {
            index: index,
            file_path: chunk_path,
            size: buffer.len(),
            encrypted_data: encrypted_data,
            nonce: nonce,
            cloud_path: None,
        });

        index += 1;
        bytes_red += read_size;
    }
    Ok(chunks)
}

fn encrypt_with_aes_siv(plain_data: &[u8]) -> (String, String) {
    let key = Aes256SivAead::generate_key(&mut OsRng);
    let cipher = Aes256SivAead::new(&key);
    let nonce = Nonce::from_slice(b"any unique nonce"); // 16 bytes; unique per message
    let encrypted_data = cipher.encrypt(nonce, plain_data.as_ref()).expect("encryption failure!");
    println!("Encrypted data: {:?}", encrypted_data);
    println!("Encrypted data (hex): {}", hex::encode(&encrypted_data));
    (hex::encode(encrypted_data), hex::encode(nonce))
}

fn hash_encrypted_data(chunk_data: &String) -> String {


    //let mut hasher = Sha256::new();
    //let mut hash_results: Vec<String> = Vec::new();
    let mut hasher = Sha256::new();
    //hasher.finalize();
    hasher.update(&chunk_data.as_bytes());
    let hash_result = hasher.finalize();
    let hash_string = format!("{:x}", hash_result);
    println!("Hash of chunk {}", hash_string);
    // fürs erste der name der Datei
    hash_string


}


fn reconstruct_file(){}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_file() {
        // if test file does not exist, create it and write some data
        use std::path::PathBuf;
        /*if !Path::new("src/test.txt").exists() {
            let mut file = File::create("src/test.txt").unwrap();
            file.write_all(b"This is a test file with some data to split.")
                .unwrap();
        }*/
        let file_path = PathBuf::from("test/test_pdf.pdf");
        let output_path = PathBuf::from("test/output_chunks");
        
        let result = split_file(file_path, output_path);
        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert!(!chunks.is_empty());
    }
    
    
    #[test]
    fn test_hash_chunk(){
        let mut chunks = vec![
            FileChunk {
                index: 0,
                file_path: PathBuf::from("chunk_0"),
                size: 4096,
                encrypted_data: String::from("exampleencrypted1"),
                nonce: String::from("examplenonce1"),
                cloud_path: None,
            },
            FileChunk {
                index: 1,
                file_path: PathBuf::from("chunk_1"),
                size: 4096,
                encrypted_data: String::from("exampleencrypted2"),
                nonce: String::from("examplenonce2"),
                cloud_path: None,
            },
        ];
        //hash_encrypted_data(&mut chunks);
        //TODO: Tests erweitern

    }

    #[test]
    fn test_encrypt_aes_siv(){
        let data = b"Example plaintext data to encrypt";
        let encrypted = encrypt_with_aes_siv(data);
    }

}
