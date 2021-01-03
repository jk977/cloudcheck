extern crate log;
extern crate env_logger;
extern crate ipnet;
extern crate serde_json;

mod sshd;
mod hosts;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use log::{info, debug};

use sshd::SshdEvent;
use hosts::{get_address_host, Host};

/**
 * Get all lines in file `path` that are sshd logs with a failed
 * login attempt.
 */
fn get_sshd_failures(path: &str) -> io::Result<Vec<SshdEvent>> {
    let mut result: Vec<SshdEvent> = Vec::new();

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for next in reader.lines() {
        let line = next?;
        debug!("Checking line: {}", line);

        if let Ok(event) = line.parse() {
            debug!("Line added");
            result.push(event)
        }
    }

    Ok(result)
}

fn main() -> io::Result<()> {
    let mut args = env::args();
    let program = args.next().expect("Expected at least 1 argument");
    let files: Vec<String> = args.collect();

    if files.is_empty() {
        panic!("Must provide files to examine");
    }

    env_logger::init();
    info!("Starting program with name {}", program);

    for logfile in files {
        for event in get_sshd_failures(&logfile)? {
            debug!("Found event: {}", &event.log);

            match get_address_host(&event.addr) {
                Host::Unknown => (),
                t => println!("{} => {:?}", &event.addr, t),
            }
        }
    }

    Ok(())
}
