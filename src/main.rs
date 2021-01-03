extern crate log;
extern crate env_logger;
extern crate serde_json;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use log::{info, debug};

fn get_field(s: &str, n: usize) -> Option<&str> {
    s.split_whitespace().nth(n)
}

fn log_is_sshd(log: &str) -> bool {
    const SERVICE_IDX: usize = 4;
    get_field(log, SERVICE_IDX)
        .map(|s| s.starts_with("sshd["))
        .unwrap_or(false)
}

fn get_matching_lines(path: &str) -> io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for next in reader.lines() {
        const MSG_START_IDX: usize = 5;
        let line = next?;

        debug!("Checking line: {}", line);

        if log_is_sshd(&line) && get_field(&line, MSG_START_IDX) == Some("Invalid") {
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

            const IP_IDX: usize = 9;
            let ip = get_field(&line, IP_IDX).expect("Unexpected log line format");
            println!("{}", &ip);
        }
    }

    Ok(())
}
