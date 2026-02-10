use aes_gcm_siv::{
    aead::{Aead, KeyInit, OsRng},
    Aes256GcmSiv,
};
use aes_kw::Kek;
use hmac::{Hmac, Mac};
use password_hash::rand_core::RngCore;
use scrypt::{
    password_hash::{PasswordHash, PasswordVerifier},
    scrypt, Params,
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::{
    fmt::format,
    fs::File,
    io::{self, Read, Write},
};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, Zeroizing};

type HmacSha256 = Hmac<Sha256>;
const HMAC_VERIFIER_MESSAGE: &[u8] = b"kek_verification";
/* OWWASP Parameter
N= 2^15
r= 8
p=1
*/
const SCRYPT_N: u8 = 15;
const SCRYPT_R: u32 = 8;
const SCRYPT_P: u32 = 1;

//TODO: https://crates.io/crates/secrecy hinzufügen

pub fn scrypt_kek_key_derivation(password: &str, salt: [u8; 32]) -> Zeroizing<[u8; 32]> {
    //let salt = generate_salt();
    let params = Params::new(SCRYPT_N, SCRYPT_R, SCRYPT_P, 64).expect("Invalid scrypt params");
    let mut kek = Zeroizing::new([0u8; 32]);
    //Zeroize is a generic wrapper type that impls Deref and DerefMut -> also muss "*" als Dereference verwendet werden
    scrypt(password.as_bytes(), &salt, &params, &mut *kek).expect("scrypt derivation failed");
    kek
}

//32 Byte langer Salt
pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn generate_dek() -> Zeroizing<[u8; 32]> {
    let mut dek = Zeroizing::new([0u8; 32]);
    OsRng.fill_bytes(&mut *dek);
    dek
}

//
pub fn kek_salt_storage<P: AsRef<Path>>(
    salt: [u8; 32],
    kek: &[u8; 32],
    kek_salt_path: P,
) -> io::Result<()> {
    //Muster: salt[0..31]tag[32..63], wichtig fürs Buffer auslesen
    let verification_tag = kek_verifier(kek);
    let mut salt_file = File::create(kek_salt_path)?;
    salt_file.write_all(&salt)?;
    salt_file.write_all(&verification_tag)?;
    Ok(())
}
//TODO: umbenennen zu load_kek_salt_and_derive o.Ä. für bessere Verstöndlichkeit
// im workflow: lade salt und dann leite kek ab, verifiziere mit tag, welcher unter kek_salt_storage berechnet wurde
pub fn load_kek_salt<P: AsRef<Path> + ?Sized>(
    password: &str,
    kek_salt_path: &P,
) -> io::Result<Zeroizing<[u8; 32]>> {
    let mut salt_file = File::open(kek_salt_path)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "KEK nicht initialisiert."))?;

    let mut salt = [0u8; 32];
    let mut stored_tag = [0u8; 32];

    salt_file.read_exact(&mut salt)?;
    salt_file.read_exact(&mut stored_tag).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Salt File ist ungültig, entweder ist Stores Tag nicht gespeichert oder Salt File ist veraltet",
        )
    })?;

    let kek = scrypt_kek_key_derivation(password, salt);
    //constant time vergleich beider tags welche
    let verifier_tag = kek_verifier(&kek);
    if verifier_tag.ct_eq(&stored_tag).unwrap_u8() != 1 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Falsches Passwort!",
        ));
    }

    Ok(kek)
}

pub fn init_kek_salt<P: AsRef<Path>>(
    password: &str,
    kek_salt_path: P,
) -> io::Result<Zeroizing<[u8; 32]>> {
    let salt = generate_salt();
    let kek = scrypt_kek_key_derivation(password, salt);
    kek_salt_storage(salt, &kek, kek_salt_path)?;
    Ok(kek)
}

// Passwort Hash welcher für die Ver-xor-ung mit Schlüssel benutzt wird
pub fn sha256_hash_password(password: &str) -> [u8; 32] {
    let result = Sha256::digest(password.as_bytes());
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

//TODO: https://datatracker.ietf.org/doc/html/rfc3394
// https://datatracker.ietf.org/doc/html/rfc5649
// eventuell das einfache Ver-XOR-en des Keys mit Passwort durch KEy Wrapping ALgorithmus ersetzen
pub fn wrap_dek_key(kek: &[u8; 32], dek: &[u8; 32]) -> io::Result<Vec<u8>> {
    let wrapper = Kek::from(*kek);
    wrapper.wrap_vec(dek).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("DEK wrap failed: {:?}", e),
        )
    })
}

pub fn unwrap_key(kek: &[u8; 32], wrapped_dek: &[u8]) -> io::Result<Zeroizing<[u8; 32]>> {
    let wrapper = Kek::from(*kek);
    let unwrapped = wrapper.unwrap_vec(wrapped_dek).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("DEK wrap failed: {:?}", e),
        )
    })?;
    let mut dek = Zeroizing::new([0u8; 32]);
    dek.copy_from_slice(&unwrapped);
    Ok(dek)
}

// einfacher workflow : https://docs.rs/hmac/latest/hmac/
pub fn kek_verifier(kek: &[u8; 32]) -> [u8; 32] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(kek).expect("HMAC accepts any key size");
    mac.update(HMAC_VERIFIER_MESSAGE);
    let result = mac.finalize();
    let mut tag = [0u8; 32];
    tag.copy_from_slice(&result.into_bytes());
    tag
}
