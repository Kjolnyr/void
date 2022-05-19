extern crate libc;

use std::{os::unix::prelude::RawFd, mem, io, ptr, time};

pub struct FdSet(libc::fd_set);

// Select code got heartlessly ripped from https://blog.pjam.me/posts/select-syscall-in-rust/

impl FdSet {
    pub fn new() -> FdSet {
        let mut raw_fd_set = mem::MaybeUninit::<libc::fd_set>::uninit();
        unsafe {
            libc::FD_ZERO(raw_fd_set.as_mut_ptr());
            FdSet(raw_fd_set.assume_init())
        }
    }

    pub fn _clear(&mut self, fd: RawFd) {
        unsafe { libc::FD_CLR(fd, &mut self.0) };
    }
    pub fn set(&mut self, fd: RawFd) {
        unsafe { libc::FD_SET(fd, &mut self.0) };
    }
    pub fn isset(&mut self, fd: RawFd) -> bool {
        unsafe { libc::FD_ISSET(fd, &mut self.0) }
    }
}

fn to_fdset_ptr(opt: Option<&mut FdSet>) -> *mut libc::fd_set {
    match opt {
        None => ptr::null_mut(),
        Some(&mut FdSet(ref mut raw_fd_set)) => raw_fd_set
    }
}

fn to_ptr<T>(opt: Option<&T>) -> *const T {
    match opt {
        None => ptr::null::<T>(),
        Some(p) => p
    }
}

pub fn make_timeval(duration: time::Duration) -> libc::timeval {
    libc::timeval {
        tv_sec: duration.as_secs() as i64,
        tv_usec: duration.subsec_micros() as i64
    }
}

pub fn select(
    nfds: libc::c_int, 
    readfds: Option<&mut FdSet>, 
    writefds: Option<&mut FdSet>,
    errorfds: Option<&mut FdSet>,
    timeout: Option<&libc::timeval>
) -> io::Result<usize> {

    match unsafe {
        libc::select(
            nfds,
            to_fdset_ptr(readfds),
            to_fdset_ptr(writefds),
            to_fdset_ptr(errorfds),
            to_ptr::<libc::timeval>(timeout) as *mut libc::timeval
        )
    } {
        -1 => Err(io::Error::last_os_error()),
        res => Ok(res as usize)
    }
}