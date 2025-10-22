use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::fs::{self,File};

pub struct FileChunk {
    pub index: usize,
    pub file_path: PathBuf,
    pub size: usize,
}

const CHUNK_SIZE: usize = 4096;

fn split_file<P: AsRef<Path>, Q: AsRef<Path>>(file_path: P, output_path: Q) -> io::Result<Vec<FileChunk>> {

    /*
    die gechnkten dateien sollen in einen ordner gespeichert werden, der name des ordners ist eine uuid
    der user sollte den pfad angeben wo der ordner erstellt werden soll


    */
    fs::create_dir_all(output_path.as_ref())?;

    let mut file = File::open(file_path.as_ref())?;
    let mut chunks: Vec<FileChunk> = Vec::new();
    let mut buffer: Vec<u8> = Vec::with_capacity(CHUNK_SIZE);

    let mut index = 0;
    // https://stackoverflow.com/questions/68694399/most-idiomatic-way-to-read-a-range-of-bytes-from-a-file
    file.read_exact(&mut buffer)?;
    let chunk_name = format!("chunk_{}", index);
    let chunk_path = output_path.as_ref().join(chunk_name);
    let mut chunk_file = File::create(&chunk_path)?;
    print!("buffer: {:?}", &buffer);
    chunk_file.write_all(&buffer)?;
    chunks.push(FileChunk {
        index,
        file_path: chunk_path,
        size: buffer.len(),
    });
    Ok(chunks)

}

fn hash_chunk(){

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_file() {
        // if test file does not exist, create it
        use std::fs;
        use std::io::Write;
        use std::path::PathBuf;
        if !Path::new("src/test.txt").exists() {
            let mut file = File::create("src/test.txt").unwrap();

        }
        let file_path = PathBuf::from("src/test.txt");
        let output_path = PathBuf::from("output_chunks");
        let result = split_file(file_path, output_path);
        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert!(!chunks.is_empty());
    }
}