use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
pub struct FileChunk {
    pub index: usize,
    pub file_path: PathBuf,
    pub size: usize,
    pub encrypted_data: String,
    pub hashed_data: String,
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
    fs::create_dir_all(output_path.as_ref())?;

    let mut file = File::open(file_path.as_ref())?;
    let mut chunks: Vec<FileChunk> = Vec::new();
    let file_size = file.metadata()?.len() as usize;
    let mut index = 0;
    let mut bytes_red = 0;

    while bytes_red < file_size {
        let rest = file_size - bytes_red;
        let read_size = std::cmp::min(rest, CHUNK_SIZE);

        let mut buffer = vec![0u8; read_size];
        //buffer needs to be hashed
        let mut hasher = Sha256::new();
        hasher.update(&buffer);
        file.read_exact(&mut buffer)?;
        //hasher.finalize();
        // fürs erste der name der Datei
        let chunk_name = format!("chunk_{}", index);
        let chunk_path = output_path.as_ref().join(chunk_name);
        // erstellen der Datei passiert erst nach dem Hashen
        let mut chunk_file = File::create(&chunk_path)?;

        chunk_file.write_all(&buffer)?;
        // TODO: metadaten index, filepath, size usw. sind nachher wichtig für Tabelle
        chunks.push(FileChunk {
            index,
            file_path: chunk_path,
            size: buffer.len(),
            encrypted_data: String::new(),
            hashed_data: String::new(),
        });

        index += 1;
        bytes_red += read_size;
    }
    Ok(chunks)
}

fn hash_encrypted_data(chunk: Vec<FileChunk>){


    //let mut hasher = Sha256::new();
    //let mut hash_results: Vec<String> = Vec::new();
    for c in chunk.iter(){
        let mut hasher = Sha256::new();
        //hasher.finalize();
        hasher.update(&c.encrypted_data.as_bytes());
        let hash_result = hasher.finalize();
        let hash_string = format!("{:x}", hash_result);
        println!("Hash of chunk {}", hash_string);
        // fürs erste der name der Datei


    }

}

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
        if Path::new(&output_path).exists() {
            fs::remove_dir_all(&output_path).ok();
        }

        let result = split_file(file_path, output_path);
        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert!(!chunks.is_empty());
    }
    
    
    #[test]
    fn test_hash_chunk(){
        let chunks = vec![
            FileChunk {
                index: 0,
                file_path: PathBuf::from("chunk_0"),
                size: 4096,
                encrypted_data: String::from("exampleencrypted1"),
                hashed_data: String::from("examplehash1"),
            },
            FileChunk {
                index: 1,
                file_path: PathBuf::from("chunk_1"),
                size: 4096,
                encrypted_data: String::from("exampleencrypted2"),
                hashed_data: String::from("examplehash2"),
            },
        ];
        hash_encrypted_data(chunks);
        //TODO: Tests erweitern

    }



}
