use libc::{self, c_int};
use std::{fs::File, os::unix::prelude::*, process};

pub mod clone;
pub mod futex;

fn pipe() -> (File, File) {
    let mut fds: [c_int; 2] = [0; 2];
    let res = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if res != 0 {
        process::exit(1);
    }
    let reader = unsafe { File::from_raw_fd(fds[0]) };
    let writer = unsafe { File::from_raw_fd(fds[1]) };
    (reader, writer)
}

fn fcntl_setfd(file: &mut File, flags: c_int) {
    let res = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags) };
    if res != 0 {
        println!("Non-zero fcntl return {:?}", res);
    }
}
