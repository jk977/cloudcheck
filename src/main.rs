extern crate log;
extern crate env_logger;
extern crate json;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use log::{info, debug, log_enabled, Level};

fn get_field(s: &str, n: usize) -> Option<&str> {
    s.split_whitespace().nth(n)
}

fn log_is_sshd(log: &str) -> bool {
    get_field(log, 4)
        .map(|s| s.starts_with("sshd["))
        .unwrap_or(false)
}

fn get_matching_lines(path: &str) -> io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for next in reader.lines() {
        let line = next?;
        debug!("Checking line: {}", line);

        if log_is_sshd(&line) && get_field(&line, 5) == Some("Invalid") {
            debug!("Line added");
            result.push(line.clone())
        }
    }

    Ok(result)
}

fn main() -> io::Result<()> {
    let mut args = env::args();
    let program = args.next().expect("Expected at least 1 argument");

    env_logger::init();

    info!("Starting program with name {}", program);

    for logfile in args {
        for line in get_matching_lines(&logfile)? {
            debug!("Processing line: {}", &line);
            let ip = get_field(&line, 9).expect("Unexpected log line format");
            println!("{}", &ip);
        }
    }

    Ok(())
}
