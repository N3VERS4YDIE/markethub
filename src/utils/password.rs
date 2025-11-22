use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!(e.to_string()))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(hash).map_err(|e| anyhow!(e.to_string()))?;
    let result = Argon2::default().verify_password(password.as_bytes(), &parsed);
    Ok(result.is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashing_produces_unique_value_and_verifies() {
        let password = "CorrectHorseBatteryStaple";
        let hash = hash_password(password).expect("hashing should succeed");

        assert_ne!(hash, password);
        assert!(verify_password(password, &hash).expect("verification should work"));
    }

    #[test]
    fn verify_rejects_invalid_password() {
        let password = "SuperSecret";
        let hash = hash_password(password).expect("hashing should succeed");

        assert!(!verify_password("WrongPassword", &hash).expect("verification should work"));
    }
}
