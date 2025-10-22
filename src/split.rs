use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::fs::File;

pub struct FileChunk {
    pub index: usize,
    pub file_path: PathBuf,
    pub size: usize,
}

const CHUNK_SIZE: usize = 4096;

fn split_file<P: AsRef<PathBuf>>(file_path: P,output_path: P) -> io::Result<Vec<FileChunk>> {

    /*
    die gechnkten dateien sollen in einen ordner gespeichert werden, der name des ordners ist eine uuid
    der user sollte den pfad angeben wo der ordner erstellt werden soll


    */
    let mut file = File::open(file_path.as_ref())?;
    let mut chunks: Vec<FileChunk> = Vec::new();
    let mut buffer: Vec<u8> = Vec::with_capacity(CHUNK_SIZE);

    let mut index = 0;
    loop {
        
    }
    Ok(chunks)

}

fn hash_chunk(){


}

#[cfg(test)]
mod tests {}