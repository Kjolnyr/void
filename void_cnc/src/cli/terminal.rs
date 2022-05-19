use std::{sync::{Arc, Mutex}, io::{stdout, Write, stdin}, os::unix::prelude::RawFd};


use log::{warn, info};
use crate::{structs::cnc::Cnc, enums::command::Command};

pub fn run(cnc: &Arc<Mutex<Cnc>>) {
    let mut context: Option<RawFd> = None;

    loop {

        print!("{} ", match context {
            Some(val) => format!("{}>", val),
            None => String::from("*>")
        });
        stdout().flush().unwrap();

        let mut data = String::new();
        readline(&mut data).unwrap();
        if data == "" { continue; }

        let lowercase = data.to_lowercase();

        let items: Vec<&str> = lowercase.split(" ").collect();
        let mut args: Vec<&str> = Vec::new();
        if items.len() > 1 {
            args = items[1..].to_vec();
        };

        let cmd = match Command::try_from((items[0], args)) {
            Ok(cmd) => cmd,
            Err(e) => {
                warn!("{}", e);
                continue;
            }
        };

        match cmd.handle(&mut context, cnc) {
            Ok(_) => (),
            Err(e) => {
                warn!("{}", e);
                continue;
            }
        };
    }
}


fn readline(buf: &mut String) -> std::io::Result<usize> {
    stdin().read_line(buf)?;

    if let Some('\n')=buf.chars().next_back() {
        buf.pop();
    }
    if let Some('\r')=buf.chars().next_back() {
        buf.pop();
    }

    Ok(buf.len() as usize)
}