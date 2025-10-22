use std::path::PathBuf;


pub struct FileChunk {
    pub index: usize,
    pub file_path: PathBuf,
    pub size: usize,
}

fn split_file(){}

fn hash_chunk(){}

#[cfg(test)]
mod tests {}