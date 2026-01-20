use aes_gcm_siv::aead::OsRng;
use password_hash::rand_core::RngCore;
use scrypt::{scrypt, Params};
use sha2::{Digest, Sha256};
use std::{fs::{File}, io::{self, Read, Write}};
use std::path::{Path, PathBuf};
use zeroize::{Zeroize, Zeroizing};
/* OWWASP Parameter
N= 2^15
r= 8
p=1
*/
const SCRYPT_N : u8 = 15;
const SCRYPT_R : u32 = 8;
const SCRYPT_P : u32 = 1;

//TODO: https://crates.io/crates/secrecy hinzufügen

pub fn scrypt_key_derivation(password: &str, salt : [u8;32]) -> ([u8; 32], [u8; 32]) {
    //let salt = generate_salt();
    let params = Params::new(SCRYPT_N, SCRYPT_R, SCRYPT_P, 64).expect("Invalid scrypt params");
    let mut key = Zeroizing::new([0u8; 32]);
    //Zeroize is a generic wrapper type that impls Deref and DerefMut -> also muss "*" als Dereference verwendet werden
    scrypt(password.as_bytes(), &salt, &params, &mut *key).expect("scrypt derivation failed");
    (*key, salt)
}

//32 Byte langer Salt
pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn initialize_master_key(password: &str) -> ([u8; 32], [u8; 32]){
    //derive key, hash_passwortd, dann zur sicherung ver-xor-ren


    let salt = generate_salt();
    let (key, salt) = scrypt_key_derivation(password, salt);
    let password_hash = sha256_hash_password(password);
    let key_xor = xor_key_password_hash(&key, &password_hash);

    // TODO:zeroization einfügen,, nachdem key setup erfolgt ist
    //gibt den salt und passwort Hash zurück welche in der key_file gespeichert werden
    (key_xor, salt)
}

// Passwort Hash welcher für die Ver-xor-ung mit Schlüssel benutzt wird
pub fn sha256_hash_password(password: &str) -> [u8; 32]{
    let result = Sha256::digest(password.as_bytes());
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}


pub fn xor_key_password_hash<'a>(key : &'a [u8; 32], password_hash: &'a [u8; 32]) -> [u8; 32]{
    let mut key_xor = [0u8; 32];
    for i in 0..64 {
        key_xor[i] = key[i] ^ password_hash[i % 32];
    }
    key_xor
}

// vorerst in Datei welche im gleichen Ordner liegt, später vielleicht unter /etc/shadow
pub fn store_key<P: AsRef<Path>>(salt: [u8; 32], key_xor : [u8; 64], key_file_path : P)-> io::Result<()>{
    let mut key_file = File::create(key_file_path)?;
    key_file.write_all(&salt)?;
    key_file.write_all(&key_xor)?;

    Ok(())
}


//TODO: https://datatracker.ietf.org/doc/html/rfc3394
// https://datatracker.ietf.org/doc/html/rfc5649
// eventuell das einfache Ver-XOR-en des Keys mit Passwort durch KEy Wrapping ALgorithmus ersetzen
pub fn wrap_key(){}

pub fn unwrap_key(){}


// xor_key daten und salt in [u8] speichern und xor-en
pub fn load_and_unlock_key<P: AsRef<Path>>(key_file_path : P, password: &str) -> io::Result< [u8;32]>{
    let mut key_file = File::open(key_file_path)?;
    //vorerst read_exact um die erstem 32 Byte (salt) und dann die nächsten 64 Byte in die [u8] Arrays reinzulesen
    let mut salt =  [0u8; 32];

    let mut xored_key = [0u8; 32];
    
    key_file.read_exact(&mut salt)?;
    key_file.read_exact(&mut xored_key)?;
    let hashed_password = sha256_hash_password(password);
    let (mut derived_key, _) = scrypt_key_derivation(password, salt);
    let unlocked_key = xor_key_password_hash(&xored_key, &hashed_password);
    //TODO: gibt es einen besseren Weg um Korrektheit zu testen
    if derived_key != unlocked_key {
        derived_key.zeroize();
    }

    Ok(unlocked_key)
}
