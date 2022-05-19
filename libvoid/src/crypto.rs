use std::num::Wrapping;

use ring::{digest, aead::{self, OpeningKey, BoundKey, SealingKey}};

// My not so secure crypto code I guess, Didn't they say don't roll your own crypto ?
pub struct VoidNonce {
    current: Wrapping<u128>,
    start: u128,
}

// 
impl VoidNonce {
    fn new(start: [u8; 12]) -> Self {
        let mut array = [0; 16];
        array[..12].copy_from_slice(&start);
        let start = u128::from_le_bytes(array);
        Self {
            current: Wrapping(start),
            start
        }
    }
}

impl aead::NonceSequence for VoidNonce {
    fn advance(&mut self) -> Result<aead::Nonce, ring::error::Unspecified> {
        let n = self.current.0;
        self.current += 1;
        if self.current.0 == self.start {
            return Err(ring::error::Unspecified);
        }
        Ok(aead::Nonce::assume_unique_for_key(n.to_le_bytes()[..12].try_into().unwrap()))
    }
}

pub struct Crypto {
    helo_key: u32,
    opening_key: Option<aead::OpeningKey<VoidNonce>>,
    sealing_key: Option<aead::SealingKey<VoidNonce>>
}


impl Crypto {
    pub fn new(helo_key: u32) -> Self {
        Crypto{
            helo_key,
            opening_key: None,
            sealing_key: None
        }
    }

    pub fn get_helo(&self) -> &u32 {
        &self.helo_key
    }

    pub fn get_helo_as_vec(&self) -> Vec<u8> {
        vec![
            ((&self.helo_key >> 24) & 0xff) as u8,
            ((&self.helo_key >> 16) & 0xff) as u8,
            ((&self.helo_key >> 8) & 0xff) as u8,
            (&self.helo_key & 0xff) as u8
        ]
    }

    pub fn set_opening_key(&mut self, bytes: [u8; digest::SHA256_OUTPUT_LEN], nonce_seed: [u8; 12]) {
        let key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &bytes).unwrap();
        let nonce_gen = VoidNonce::new(nonce_seed);
        self.opening_key = Some(OpeningKey::new(key, nonce_gen))
    }

    pub fn set_sealing_key(&mut self, bytes: [u8; digest::SHA256_OUTPUT_LEN], nonce_seed: [u8; 12]) {
        let key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &bytes).unwrap();
        let nonce_gen = VoidNonce::new(nonce_seed);
        self.sealing_key = Some(SealingKey::new(key, nonce_gen))
    }

    pub fn get_sealing_key(&mut self) -> Result<&mut SealingKey<VoidNonce>, ()> {
        match &mut self.sealing_key {
            Some(key) => Ok(key),
            None => Err(())
        }
    }

    pub fn get_opening_key(&mut self) -> Result<&mut OpeningKey<VoidNonce>, ()> {
        match &mut self.opening_key {
            Some(key) => Ok(key),
            None => Err(())
        }
    }

    pub fn reset(&mut self) {
        self.opening_key = None;
        self.sealing_key = None;
    }

}