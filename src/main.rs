extern crate pam;
use std::fs::File;
use std::io::{prelude::*};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::{fs, env};

use chrono::Utc;

const ART: &str = "
██████                 ██████  ███████ ████████ ████████ ██    ██ 
██   ██               ██       ██         ██       ██     ██  ██  
██████      █████     ██   ███ █████      ██       ██      ████   
██   ██               ██    ██ ██         ██       ██       ██    
██   ██                ██████  ███████    ██       ██       ██  
";

fn read_input(tty: &mut File, mask: Option<char>) -> String {
    let mut chars = Vec::new();
    let mut buff = [0;1];

    loop {

        tty.read(&mut buff).expect("Failed to read");
        let c = buff[0] as char;
        if c == '\n' {
            break;
        } else {
            //this does not work, unfortunately
            if let Some(m) = mask {
                write!(tty, "\x1b[1D{}", m).unwrap();
            };

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

/**
 * resources
 * https://tldp.org/HOWTO/Bash-Prompt-HOWTO/x361.html
 * https://www.tutorialspoint.com/how-to-output-colored-text-to-a-linux-terminal
 */

fn main() {
    let nr_tty: i32 = env::var("TTY")
        .expect("TTY not specified")
        .parse().expect("TTY is not a number, must be 1-12");

    let dont_clear_screen = env::var("NOCLEAR").is_ok();

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
    writeln!(tty, "\x1b[44m\x1b[1;37m{} Welcome to r-getty v03! (tty{})\x1b[0m", time, nr_tty).unwrap();


    write!(tty, "Username: ").unwrap();
    tty.flush().unwrap();
    
    let username = read_input(&mut tty, None);
    login(tty, &username, true);
}