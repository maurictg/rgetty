extern crate pam;
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{env, fs, thread};

mod pass_reader;
mod images;

use chrono::Utc;
use pass_reader::read_password;
use images::{MOTD_ART, ERR_ART};

const SERVICE: &str = "system-auth";

fn main() {
    let nr_tty: i32 = env::var("TTY")
        .expect("TTY not specified")
        .parse()
        .expect("TTY is not a number, must be 1-12");

    let dont_clear_screen = env::var("NOCLEAR").is_ok();
    let use_system_login = env::var("SYSTEMLOGIN").is_ok();
    let use_error_art = env::var("ERRORART").is_ok();
    let use_clear_delay = env::var("CLEARDELAY")
        .map(|x| x.parse::<i32>().expect("CLEARDELAY must be a number in ms"));
    let default_user_name = env::var("USERNAME");

    let ascii_art_file = env::var("ART");

    let error_delay = env::var("ERRORDELAY")
        .map(|x| x.parse::<i32>().expect("ERRORDELAY must me a number"))
        .unwrap_or(1500);

    if nr_tty < 1 || nr_tty > 12 {
        panic!("TTY number is too large");
    }

    let mut tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("/dev/tty{}", nr_tty))
        .expect("Failed to open TTY");

    if !dont_clear_screen {
        if let Ok(cd) = use_clear_delay {
            thread::sleep(Duration::from_millis(cd as u64));
        }

        write!(&mut tty, "{esc}[2J{esc}[1;1H", esc = 27 as char).expect("Could not clear TTY");
    }

    let art = if let Ok(Ok(f)) = ascii_art_file.map(|r| fs::read_to_string(r)) {
        f
    } else {
        MOTD_ART.to_owned()
    };

    writeln!(tty, "\x1b[1;32m{}\x1b[0m", art).unwrap();

    let time = Utc::now().format("%H:%M:%S");
    writeln!(
        tty,
        "\x1b[44m\x1b[1;37m{} Welcome to r-getty v04! (tty{})\x1b[0m",
        time, nr_tty
    )
    .unwrap();

    let default_user_name = default_user_name.as_ref();
    if default_user_name.is_ok() {
        write!(tty, "Username [{}]: ", default_user_name.unwrap()).unwrap();
    } else {
        write!(tty, "Username: ").unwrap();
    }

    tty.flush().unwrap();

    let mut username = read_input(&mut tty);
    if username.len() == 0 {
        username = default_user_name.unwrap().to_owned();
    }

    if use_system_login {
        login(tty, &username, true);
    } else {
        ask_and_validate_pass(&username, tty, error_delay, use_error_art);
    }
}

fn read_input(tty: &mut File) -> String {
    let mut chars = Vec::new();
    let mut buff = [0; 1];

    loop {
        tty.read(&mut buff).expect("Failed to read");
        let c = buff[0] as char;
        if c == '\n' {
            break;
        } else {
            chars.push(c);
        }
    }

    let str: String = chars.iter().collect();
    let str = str.trim();
    return str.to_owned();
}

fn login(tty: File, username: &str, ask_for_pass: bool) {
    Command::new("/bin/login")
        .stdin(Stdio::from(tty.try_clone().unwrap()))
        .stdout(Stdio::from(tty))
        .arg(username)
        .arg(if ask_for_pass { "" } else { "-f" })
        .exec();
}

fn ask_and_validate_pass(username: &str, tty: File, err_delay: i32, use_err_art: bool) {
    let mut tty = tty;
    write!(tty, "Password: ").unwrap();
    tty.flush().unwrap();

    let password = read_password(&mut tty);

    let mut auth =
        pam::Authenticator::with_password(SERVICE).expect("Failed to start the PAM service");

    auth.get_handler().set_credentials(username, &password);

    let authenticate = auth.authenticate();
    let session = auth.open_session();

    if authenticate.is_ok() && session.is_ok() {
        write!(tty, "\x1b[1;32m").unwrap();
        writeln!(tty, "Authentication SUCCEED :)").unwrap();
        write!(tty, "\x1b[0m").unwrap(); //reset color

        login(tty, username, false);
    } else {
        write!(tty, "\x1b[1;31m").unwrap();
        if use_err_art {
            writeln!(tty, "{}", ERR_ART).unwrap();
        }
        
        writeln!(tty, "Authentication FAILED :(").unwrap();

        if authenticate.is_err() {
            writeln!(
                tty,
                "Auth failure: {}",
                authenticate.err().unwrap()
            )
            .unwrap();
        }

        if session.is_err() {
            writeln!(
                tty,
                "Session failure: {}",
                session.err().unwrap()
            )
            .unwrap();
        }

        write!(tty, "\x1b[0m").unwrap(); //reset color
        tty.flush().unwrap();

        thread::sleep(Duration::from_millis(err_delay as u64));
        std::process::exit(1);
    }
}
