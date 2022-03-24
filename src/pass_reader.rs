use std::{mem, fs::File, io::{self, BufRead}, os::unix::prelude::IntoRawFd};
use libc::{tcsetattr, ECHO, ECHONL, TCSANOW, termios, c_int};

fn io_result(ret: c_int) -> ::std::io::Result<()> {
    match ret {
        0 => Ok(()),
        _ => Err(::std::io::Error::last_os_error()),
    }
}

fn safe_tcgetattr(fd: c_int) -> ::std::io::Result<termios> {
    let mut term = mem::MaybeUninit::<termios>::uninit();
    io_result(unsafe { ::libc::tcgetattr(fd, term.as_mut_ptr()) })?;
    Ok(unsafe { term.assume_init() })
}

pub fn read_password(tty: &mut File) -> String {
    let fd = tty.try_clone().unwrap().into_raw_fd();
    let mut term = safe_tcgetattr(fd).expect("Failed to create termios");
    let term_orig = safe_tcgetattr(fd).expect("Failed to get terminal");

    //hide the password
    term.c_lflag &= !ECHO;

    //don't hide newline character from 'enter
    term.c_lflag |= ECHONL;

    //save setting
    io_result(unsafe { tcsetattr(fd, TCSANOW, &term) }).expect("Failed to safe settings");

    let mut pass = String::new();
    let mut reader = io::BufReader::new(tty);
    reader
        .read_line(&mut pass)
        .expect("Failed to read password");

        //set terminal back to normal mode (text)
        unsafe {
            tcsetattr(fd, TCSANOW, &term_orig);
        }

    pass.trim_end_matches('\n').to_owned()
}