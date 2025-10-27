use std::path::PathBuf;

mod split;
mod redis_db;
mod key_management;
mod cloud;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let mut redis_client = redis_db::RedisClient::create_from_env()?;
    let input_path = PathBuf::from("test/test_pdf.pdf");
    let output_path = PathBuf::from("test/output_chunks");
    let chunk_vector = split::split_file(input_path.as_path(), output_path)?;

    let key = &chunk_vector.file_name;
    redis_client.set_hvalue(&key, &chunk_vector);
    
    Ok(())
}
