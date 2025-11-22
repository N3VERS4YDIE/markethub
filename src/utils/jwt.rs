use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct JwtConfig {
    secret: String,
    expiration: Duration,
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
}

impl JwtConfig {
    pub fn new(secret: impl Into<String>, expiration_hours: i64) -> Self {
        let secret = secret.into();
        let encoding = EncodingKey::from_secret(secret.as_bytes());
        let decoding = DecodingKey::from_secret(secret.as_bytes());
        let mut validation = Validation::default();
        validation.validate_exp = true;

        Self {
            secret,
            expiration: Duration::hours(expiration_hours.max(1)),
            encoding,
            decoding,
            validation,
        }
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn expiration(&self) -> Duration {
        self.expiration
    }

    pub fn generate(&self, claims: &Claims) -> jsonwebtoken::errors::Result<String> {
        jsonwebtoken::encode(&Header::default(), claims, &self.encoding)
    }

    pub fn claims_for(&self, user_id: Uuid, email: String) -> Claims {
        let now = Utc::now();
        let exp = now + self.expiration;
        Claims {
            sub: user_id,
            email,
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        }
    }

    pub fn verify(&self, token: &str) -> jsonwebtoken::errors::Result<Claims> {
        let token_data = jsonwebtoken::decode::<Claims>(token, &self.decoding, &self.validation)?;
        Ok(token_data.claims)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub iat: usize,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_and_verifies_tokens() {
        let config = JwtConfig::new("test-secret", 1);
        let user_id = Uuid::new_v4();
        let claims = config.claims_for(user_id, "alice@example.com".into());

        let token = config.generate(&claims).expect("token should generate");
        assert!(!token.is_empty());

        let verified = config.verify(&token).expect("token should verify");
        assert_eq!(verified.sub, user_id);
        assert_eq!(verified.email, "alice@example.com");
        assert!(verified.exp >= verified.iat);
    }
}
