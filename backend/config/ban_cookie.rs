use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
    str::FromStr,
    sync::LazyLock,
};

use chrono::Utc;
use regex;
use serde::{Deserialize, Serialize};
use snafu::{GenerateImplicitData, Location};

use crate::error::ClewdrError;

const PLACEHOLDER_COOKIE: &str = "sk-ant-sid01----------------------------SET_YOUR_COOKIE_HERE----------------------------------------AAAAAAAA";

/// A struct representing a cookie string
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClewdrCookie {
    inner: String,
}

impl Serialize for ClewdrCookie {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ClewdrCookie {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ClewdrCookie::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Simplified cookie for ban operations - only essential fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanCookie {
    pub cookie: ClewdrCookie,
    pub submitted_at: Option<String>,
    pub last_used_at: Option<String>,
    #[serde(default)]
    pub requests_sent: u64,
    #[serde(default)]
    pub is_banned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl PartialEq for BanCookie {
    fn eq(&self, other: &Self) -> bool {
        self.cookie == other.cookie
    }
}

impl Eq for BanCookie {}

impl Hash for BanCookie {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cookie.hash(state);
    }
}

impl BanCookie {
    pub fn new(cookie: &str) -> Result<Self, ClewdrError> {
        let cookie = ClewdrCookie::from_str(cookie)?;
        Ok(Self {
            cookie,
            submitted_at: Some(Utc::now().to_rfc3339()),
            last_used_at: None,
            requests_sent: 0,
            is_banned: false,
            error_message: None,
        })
    }

    pub fn mark_used(&mut self) {
        self.last_used_at = Some(Utc::now().to_rfc3339());
        self.requests_sent += 1;
    }

    pub fn mark_banned(&mut self) {
        self.is_banned = true;
    }
}

impl Deref for ClewdrCookie {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Default for ClewdrCookie {
    fn default() -> Self {
        Self {
            inner: PLACEHOLDER_COOKIE.to_string(),
        }
    }
}

impl ClewdrCookie {
    pub fn ellipse(&self) -> String {
        let len = self.inner.len();
        if len > 10 {
            format!("{}...", &self.inner[..10])
        } else {
            self.inner.to_owned()
        }
    }
}

impl FromStr for ClewdrCookie {
    type Err = ClewdrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"(?:sk-ant-sid01-)?([0-9A-Za-z_-]{86}-[0-9A-Za-z_-]{6}AA)")
                .map_err(|e| {
                    tracing::error!("Failed to compile cookie regex: {}", e);
                    ClewdrError::InternalServerError {
                        msg: format!("Regex compilation failed: {}", e),
                    }
                })
                .unwrap_or_else(|_| {
                    // Fallback regex if the main one fails
                    regex::Regex::new(r"sk-ant-[A-Za-z0-9_-]+").unwrap()
                })
        });

        let cleaned = s
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>();

        if let Some(captures) = RE.captures(&cleaned) {
            if let Some(cookie_match) = captures.get(1) {
                return Ok(Self {
                    inner: cookie_match.as_str().to_string(),
                });
            }
        }

        Err(ClewdrError::ParseCookieError {
            loc: Location::generate(),
            msg: "Invalid cookie format".into(),
        })
    }
}

impl Display for ClewdrCookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sk-ant-sid01-{}", self.inner)
    }
}

impl Debug for ClewdrCookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sk_cookie_from_str() {
        let cookie = ClewdrCookie::from_str("sk-ant-sid01----------------------------SET_YOUR_COOKIE_HERE----------------------------------------AAAAAAAA")
            .expect("Valid cookie should parse successfully");
        assert_eq!(cookie.inner.len(), 95);
    }

    #[test]
    fn test_cookie_from_str() {
        let cookie = ClewdrCookie::from_str("dif---------------------------SET_YOUR_COOKIE_HERE----------------------------------------AAAAAAAAdif")
            .expect("Valid cookie should parse successfully");
        assert_eq!(cookie.inner.len(), 95);
    }

    #[test]
    fn test_invalid_cookie() {
        let result = ClewdrCookie::from_str("invalid-cookie");
        assert!(result.is_err());
    }
}
