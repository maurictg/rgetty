extern crate pam;
use std::fs::File;
use std::io::{prelude::*};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{env, fs, thread};

mod pass_reader;

use chrono::Utc;
use pass_reader::read_password;

const SERVICE: &str = "system-auth";
const ART: &str = "
██████                 ██████  ███████ ████████ ████████ ██    ██ 
██   ██               ██       ██         ██       ██     ██  ██  
██████      █████     ██   ███ █████      ██       ██      ████   
██   ██               ██    ██ ██         ██       ██       ██    
██   ██                ██████  ███████    ██       ██       ██  
";

fn read_input(tty: &mut File) -> String {
    let mut chars = Vec::new();
    let mut buff = [0; 1];

    loop {
        tty.read(&mut buff).expect("Failed to read");
        let c = buff[0] as char;
        if c == '\n' {
            break;
        } else {
            //this does not work, unfortunately
            /*if let Some(m) = mask {
                write!(tty, "\x1b[1D{}", m).unwrap();
            };*/

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

fn main() {
    let nr_tty: i32 = env::var("TTY")
        .expect("TTY not specified")
        .parse()
        .expect("TTY is not a number, must be 1-12");

    let dont_clear_screen = env::var("NOCLEAR").is_ok();
    let use_system_login = env::var("SYSTEMLOGIN").is_ok();

    if nr_tty < 1 || nr_tty > 12 {
        panic!("TTY number is too large");
    }

    let mut tty = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("/dev/tty{}", nr_tty))
        .expect("Failed to open TTY");

    if !dont_clear_screen {
        write!(&mut tty, "{esc}[2J{esc}[1;1H", esc = 27 as char).expect("Could not clear TTY");
    }

    writeln!(tty, "\x1b[1;32m{}\x1b[0m", ART).unwrap();

    let time = Utc::now().format("%H:%M:%S");
    writeln!(
        tty,
        "\x1b[44m\x1b[1;37m{} Welcome to r-getty v04! (tty{})\x1b[0m",
        time, nr_tty
    )
    .unwrap();

    write!(tty, "Username: ").unwrap();
    tty.flush().unwrap();

    let username = read_input(&mut tty);
    
    if use_system_login {
        login(tty, &username, true);
    } else {
        ask_and_validate_pass(&username, tty);
    }
}

fn ask_and_validate_pass(username: &str, tty: File) {
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
        writeln!(tty, "Authentication FAILED :(").unwrap();

        if authenticate.is_err() {
            writeln!(
                tty,
                "Auth failure: {}",
                authenticate.err().unwrap().to_string()
            )
            .unwrap();
        }

        if session.is_err() {
            writeln!(
                tty,
                "Session failure: {}",
                session.err().unwrap().to_string()
            )
            .unwrap();
        }

        write!(tty, "\x1b[0m").unwrap(); //reset color
        tty.flush().unwrap();
        
        thread::sleep(Duration::from_millis(2000));
        std::process::exit(1);
    }
}