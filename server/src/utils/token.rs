use anyhow::{anyhow, Context};

use pikpak_core::api::Ident;
use ring::{
    aead::{self, Aad, BoundKey, Nonce, NonceSequence, NONCE_LEN},
    error::Unspecified,
    rand::SecureRandom,
};
use serde::{Deserialize, Serialize};

const AES_256_GCM_LEN: usize = 32;

pub(crate) struct Cipher {
    key: [u8; AES_256_GCM_LEN],
}

#[derive(Clone)]
pub(crate) struct NonceSeq([u8; NONCE_LEN]);

impl NonceSequence for &mut NonceSeq {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        Nonce::try_assume_unique_for_key(&self.0)
    }
}

lazy_static::lazy_static! {
    pub(crate) static ref NONCE: NonceSeq = {
        let rng = ring::rand::SystemRandom::new();
        let mut nonce = [0u8; NONCE_LEN];
        rng.fill(&mut nonce).expect("gen nonce error");
        NonceSeq(nonce)
    };
}

impl Cipher {
    pub(crate) fn new() -> Self {
        let rng = ring::rand::SystemRandom::new();
        let mut key = [0u8; AES_256_GCM_LEN];
        rng.fill(&mut key).expect("gen key error");

        Self { key }
    }

    pub(crate) fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &self.key)
            .map_err(|e| anyhow!(e))
            .context("[cipher encrypt] create key error")?;

        let nonce = &mut NONCE.clone();
        let mut sealing_key = aead::SealingKey::new(unbound_key, nonce);

        let mut data = data.to_vec();

        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut data)
            .map_err(|e| anyhow!(e))
            .context("[cipher encrypt] encrypt data error")?;
        Ok(data)
    }

    pub(crate) fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &self.key)
            .map_err(|e| anyhow!(e))
            .context("[cipher decrypt] create key error")?;

        let nonce = &mut NONCE.clone();
        let mut open_key = aead::OpeningKey::new(unbound_key, nonce);

        let mut data = data.to_vec();
        open_key
            .open_in_place(Aad::empty(), &mut data)
            .map_err(|e| anyhow!(e))
            .context("[cipher decrypt] decrypt data error")
            .map(|x| x.to_vec())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TokenData {
    pub email: String,
    pub password: String,
}

impl From<TokenData> for Ident {
    fn from(val: TokenData) -> Self {
        Ident {
            username: val.email,
            password: val.password,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cipher() {
        let cipher = Cipher::new();
        let data = b"hello world";
        let encrypted = cipher.encrypt(data).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(data, decrypted.as_slice());
    }
}
