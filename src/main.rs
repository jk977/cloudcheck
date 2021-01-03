extern crate log;
extern crate env_logger;
extern crate ipnet;
extern crate serde_json;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::net::Ipv4Addr;

use ipnet::Ipv4Net;
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

fn extract_log_ip(log: &str) -> Option<Ipv4Addr> {
    const ADDR_IDX: usize = 9;
    get_field(&log, ADDR_IDX)?
        .parse()
        .ok()
}

macro_rules! gen_json_parser {
    {$name: ident, $ip_field: literal} => {
        fn $name(path: &str) -> io::Result<Vec<Ipv4Net>> {
            let file = File::open(path)?;
            let json: serde_json::Value = serde_json::from_reader(file)?;
            let prefixes = json["prefixes"]
                .as_array()
                .expect("Unrecognized JSON format");

            let mut result: Vec<Ipv4Net> = Vec::new();

            for info in prefixes {
                if let Some(prefix) = info[$ip_field].as_str() {
                    let net: Ipv4Net = prefix.parse().expect("Failed to parse IPv4 prefix");
                    result.push(net);
                }
            }

            Ok(result)
        }
    }
}

gen_json_parser!{parse_google_json, "ipv4Prefix"}
gen_json_parser!{parse_aws_json, "ip_prefix"}

fn addr_in_networks(addr: &Ipv4Addr, networks: &[Ipv4Net]) -> bool {
    for network in networks {
        if network.contains(addr) {
            return true;
        }
    }

    false
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

    let google_networks = parse_google_json("data/google-cloud-ranges.json")?;
    let aws_networks = parse_aws_json("data/aws-ranges.json")?;

    for logfile in files {
        for line in get_matching_lines(&logfile)? {
            debug!("Processing line: {}", &line);

            let addr = extract_log_ip(&line).expect("Failed to parse log IP");

            if addr_in_networks(&addr, &google_networks) {
                println!("Google address: {}", addr);
            } else if addr_in_networks(&addr, &aws_networks) {
                println!("AWS address: {}", addr);
            }
        }
    }

    Ok(())
}
