use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPasswordHash(pub String);

pub fn password_meets_policy(plain: &str) -> bool {
    plain.len() >= 12
}

pub fn hash_password(plain: &str) -> Result<StoredPasswordHash> {
    if !password_meets_policy(plain) {
        return Err(Error::Crypto("password too short".into()));
    }
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(StoredPasswordHash(hash.to_string()))
}

pub fn verify_password(plain: &str, hash: &StoredPasswordHash) -> Result<bool> {
    let parsed = PasswordHash::new(&hash.0).map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_password() {
        assert!(!password_meets_policy("short"));
    }

    #[test]
    fn hash_verify_roundtrip() {
        let h = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password("correct horse battery staple", &h).unwrap());
        assert!(!verify_password("wrong password!!", &h).unwrap());
    }
}
