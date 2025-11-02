use std::{io::Cursor, marker::PhantomData};

use actix_web::{ResponseError, http::StatusCode};
use argon2::Argon2;
use chacha20poly1305::{ChaCha20Poly1305, ChaChaPoly1305, KeyInit, Nonce, aead::Aead};
use serde::{Serialize, de::DeserializeOwned};

pub struct CryptoSuite {
    key: ChaCha20Poly1305,
    nonce: Nonce,
}

impl CryptoSuite {
    pub fn new(password: &str, salt: &str) -> argon2::Result<Self> {
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        let mut buf = [0; 44];
        Argon2::default().hash_password_into(password.as_bytes(), salt.as_bytes(), &mut buf)?;
        key.copy_from_slice(&buf[..32]);
        nonce.copy_from_slice(&buf[32..]);
        let key = ChaChaPoly1305::new(&key.into());
        let nonce = Nonce::from(nonce);
        Ok(Self { key, nonce })
    }
}

#[derive(Debug, sqlx::Type, serde::Deserialize, serde::Serialize, Clone)]
pub struct CryptData<T> {
    data: Vec<u8>,
    _pd: PhantomData<T>,
}

impl<T> From<Vec<u8>> for CryptData<T> {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: value,
            _pd: PhantomData,
        }
    }
}

impl<T> Into<Vec<u8>> for CryptData<T> {
    fn into(self) -> Vec<u8> {
        self.data
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptError {
    #[error(transparent)]
    CiborSer(#[from] ciborium::ser::Error<std::io::Error>),
    #[error(transparent)]
    CiborDer(#[from] ciborium::de::Error<std::io::Error>),
    #[error("Crypto error: {0}")]
    ChaCha(chacha20poly1305::Error),
}

impl ResponseError for CryptError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<chacha20poly1305::Error> for CryptError {
    fn from(value: chacha20poly1305::Error) -> Self {
        Self::ChaCha(value)
    }
}

impl<T: Serialize + DeserializeOwned> CryptData<T> {
    pub fn encrypt(data: T, suite: &CryptoSuite) -> Result<Self, CryptError> {
        let mut buf = Vec::new();
        ciborium::into_writer(&data, &mut buf)?;
        let data = suite.key.encrypt(&suite.nonce, buf.as_slice())?;
        Ok(Self {
            data,
            _pd: PhantomData,
        })
    }
    pub fn decrypt(self, suite: &CryptoSuite) -> Result<T, CryptError> {
        let buf = suite.key.decrypt(&suite.nonce, self.data.as_slice())?;
        Ok(ciborium::de::from_reader(Cursor::new(buf))?)
    }
}
