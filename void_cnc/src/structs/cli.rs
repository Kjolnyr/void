use std::{sync::{Mutex}, io::{stdout, stdin, Write}};
use libvoid::errors::*;
use super::{cnc::Cnc};

pub struct CncCli<'a> {
    context: Option<usize>,
    cnc: &'a Mutex<Cnc>,
}

impl<'a> CncCli<'a> {
    pub fn new(cnc: &'a Mutex<Cnc>) -> Self {
        CncCli { context: None, cnc }
    }

    pub fn get_cnc(&self) -> &Mutex<Cnc> {
        &self.cnc
    }

    pub fn get_context(&self) -> &Option<usize> {
        &self.context
    }

    pub fn set_context(&mut self, new_context: Option<usize>) {
        self.context = new_context
    }

    pub fn handle_command(&mut self) -> VoidResult<()> {

        let mut data = String::new();
        self.readline(&mut data).unwrap();

        let lowercase = data.to_lowercase();

        let items: Vec<&str> = lowercase.split(" ").collect();
        let cmd = items[0];
        let mut args: Vec<&str> = Vec::new();

        if items.len() > 1 {
            args = items[1..].to_vec();
        };

        // let cmd = Command::try_from(items[0]);
        // cmd.handle(args);
    
        Ok(())
    }

    fn readline(&self, buf: &mut String) -> std::io::Result<usize> {
        print!("{} ", self.get_ps());
        stdout().flush()?;
    
        stdin().read_line(buf)?;
    
        if let Some('\n')=buf.chars().next_back() {
            buf.pop();
        }
        if let Some('\r')=buf.chars().next_back() {
            buf.pop();
        }
    
        Ok(buf.len() as usize)
    }

    fn get_ps(&self) -> String {
        match self.context {
            Some(val) => format!("{}>", val),
            None => String::from("*>")
        }
    }
}