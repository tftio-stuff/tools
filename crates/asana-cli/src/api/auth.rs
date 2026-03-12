//! Authentication helpers for the Asana API client.

use secrecy::{ExposeSecret, SecretString};
use std::fmt;

/// Wrapper around a Personal Access Token (PAT) ensuring secret handling.
#[derive(Clone, Debug)]
pub struct AuthToken(SecretString);

impl AuthToken {
    /// Construct a new token wrapper.
    #[must_use]
    pub const fn new(token: SecretString) -> Self {
        Self(token)
    }

    /// Expose the token contents.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl From<SecretString> for AuthToken {
    fn from(value: SecretString) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted token>")
    }
}

/// Trait for providing access tokens; enables testing without touching the
/// concrete configuration layer.
pub trait TokenProvider: Send + Sync {
    /// Obtain a fresh Personal Access Token.
    fn personal_access_token(&self) -> SecretString;
}

/// Simple token provider that always returns the same token.
#[derive(Clone, Debug)]
pub struct StaticTokenProvider {
    token: AuthToken,
}

impl StaticTokenProvider {
    /// Create a new provider from a token.
    #[must_use]
    pub const fn new(token: AuthToken) -> Self {
        Self { token }
    }
}

impl TokenProvider for StaticTokenProvider {
    fn personal_access_token(&self) -> SecretString {
        SecretString::new(self.token.expose().to_owned().into())
    }
}

impl From<AuthToken> for StaticTokenProvider {
    fn from(token: AuthToken) -> Self {
        Self::new(token)
    }
}
