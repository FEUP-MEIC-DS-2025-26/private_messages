use std::{io::Cursor, marker::PhantomData, ops::Deref};

use actix_web::{ResponseError, http::StatusCode};
use argon2::Argon2;
use chacha20poly1305::{ChaCha20Poly1305, ChaChaPoly1305, KeyInit, aead::Aead};
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{Decode, Encode, Sqlite};

pub struct CryptoKey {
    key: ChaCha20Poly1305,
}

impl CryptoKey {
    pub fn new(password: &str, salt: &str) -> argon2::Result<Self> {
        let mut buf = [0; 32];
        Argon2::default().hash_password_into(password.as_bytes(), salt.as_bytes(), &mut buf)?;
        let key = ChaChaPoly1305::new(&buf.into());
        Ok(Self { key })
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
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

impl<T> From<CryptData<T>> for Vec<u8> {
    fn from(val: CryptData<T>) -> Self {
        val.data
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
    pub fn encrypt<RNG: rand::CryptoRng>(
        data: T,
        suite: &CryptoKey,
        rng: &mut RNG,
    ) -> Result<(Self, [u8; 12]), CryptError> {
        let mut buf = Vec::new();
        ciborium::into_writer(&data, &mut buf)?;
        let mut nonce_buf = [0u8; 12];
        rng.fill_bytes(&mut nonce_buf);

        let data = suite.key.encrypt(&nonce_buf.into(), buf.as_slice())?;
        Ok((
            Self {
                data,
                _pd: PhantomData,
            },
            nonce_buf,
        ))
    }
    pub fn decrypt(self, suite: &CryptoKey, nonce: &[u8; 12]) -> Result<T, CryptError> {
        let buf = suite.key.decrypt(nonce.into(), self.data.as_slice())?;
        Ok(ciborium::de::from_reader(Cursor::new(buf))?)
    }
}

impl<T> Deref for CryptData<T> {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'q, DB, T> Encode<'q, DB> for CryptData<T>
where
    DB: sqlx::Database,
    Vec<u8>: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <Vec<u8> as Encode<'q, DB>>::encode_by_ref(&self.data, buf)
    }

    fn encode(
        self,
        buf: &mut <DB as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        <Vec<u8> as Encode<'q, DB>>::encode(self.data, buf)
    }
}

impl<'r, DB, T> Decode<'r, DB> for CryptData<T>
where
    DB: sqlx::Database,
    Vec<u8>: Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(CryptData::from(<Vec<u8> as Decode<'r, DB>>::decode(value)?))
    }
}

impl<T> sqlx::Type<Sqlite> for CryptData<T> {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <Vec<u8> as sqlx::Type<Sqlite>>::type_info()
    }
}
