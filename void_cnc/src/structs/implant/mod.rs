use std::{net::{SocketAddr, TcpStream}, time::SystemTime, os::unix::prelude::{RawFd, AsRawFd}};
use libvoid::{crypto::Crypto};

pub struct Implant {
    addr: SocketAddr,
    crypto: Crypto,
    stream: Option<TcpStream>,
    writebuf: Vec<u8>,
    readbuf: Vec<u8>,
    lastping: SystemTime,
    is_init: bool
}


impl Implant {

    pub fn new(addr: SocketAddr, helo_key: u32, stream: TcpStream) -> Self {
        Self{
            addr,
            crypto: Crypto::new(helo_key),
            stream: Some(stream), 
            writebuf: Vec::new(), 
            readbuf: Vec::new(),
            lastping: SystemTime::now(),
            is_init: false}
    }

    pub fn get_fd(&self) -> RawFd {
        
        match &self.stream {
            Some(stream) => stream.as_raw_fd(),
            None => -1
        }
    }

    pub fn get_addr_str(&self) -> String {
        self.addr.to_string()
    }

    pub fn print_last_ping(&self) -> String {
        if let Ok(seconds) = self.lastping.elapsed() {
            return seconds.as_secs().to_string() + " s";
        } else { 
            "UNK".to_string() 
        }
    }
}

mod net;
mod handler;