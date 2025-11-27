use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::error::ClewdrError;

const DEFAULT_TOKEN_TTL_SECS: u64 = 60 * 30; // 30 minutes

#[derive(Clone)]
pub struct TokenManager {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
    ttl: Duration,
    issuer: String,
    #[allow(dead_code)]
    secret: Arc<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

#[derive(Debug, Clone)]
pub struct IssuedToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

impl TokenManager {
    pub fn from_env() -> Self {
        let secret_bytes = std::env::var("CLEWDR_AUTH_SECRET")
            .map(|value| value.into_bytes())
            .unwrap_or_else(|_| {
                let mut bytes = vec![0u8; 32];
                OsRng.fill_bytes(&mut bytes);
                bytes
            });

        Self::new(secret_bytes, Duration::from_secs(Self::ttl_from_env()))
    }

    fn ttl_from_env() -> u64 {
        std::env::var("CLEWDR_AUTH_TTL")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v >= 60)
            .unwrap_or(DEFAULT_TOKEN_TTL_SECS)
    }

    pub fn new(secret: Vec<u8>, ttl: Duration) -> Self {
        let encoding = EncodingKey::from_secret(&secret);
        let decoding = DecodingKey::from_secret(&secret);
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.set_required_spec_claims(&["exp", "iat", "iss", "sub"]);

        Self {
            encoding,
            decoding,
            validation,
            ttl,
            issuer: "clewdr".to_string(),
            secret: Arc::new(secret),
        }
    }

    pub fn issue(&self, subject: &str) -> Result<IssuedToken, ClewdrError> {
        let now = Utc::now();
        let expires_at = now
            + chrono::Duration::from_std(self.ttl)
                .unwrap_or_else(|_| chrono::Duration::seconds(DEFAULT_TOKEN_TTL_SECS as i64));

        let claims = TokenClaims {
            sub: subject.to_string(),
            iss: self.issuer.clone(),
            iat: now.timestamp() as usize,
            exp: expires_at.timestamp() as usize,
        };

        let token =
            jsonwebtoken::encode(&Header::default(), &claims, &self.encoding).map_err(|e| {
                ClewdrError::Whatever {
                    message: "Failed to sign authentication token".to_string(),
                    source: Some(Box::new(e)),
                }
            })?;

        Ok(IssuedToken { token, expires_at })
    }

    pub fn validate(&self, token: &str) -> Result<TokenClaims, ClewdrError> {
        let data = jsonwebtoken::decode::<TokenClaims>(token, &self.decoding, &self.validation)
            .map_err(|_| ClewdrError::InvalidAuth)?;
        Ok(data.claims)
    }
}
