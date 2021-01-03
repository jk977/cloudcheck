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

const GOOGLE_JSON: &'static str = "data/google-cloud-ranges.json";
const AWS_JSON: &'static str = "data/aws-ranges.json";

/**
 * Get the `n`th field in `s`, using whitespace as a delimiter.
 */
fn get_field(s: &str, n: usize) -> Option<&str> {
    s.split_whitespace().nth(n)
}

/**
 * Check if `log` is an sshd log.
 */
fn log_is_sshd(log: &str) -> bool {
    const SERVICE_IDX: usize = 4;
    get_field(log, SERVICE_IDX)
        .map(|s| s.starts_with("sshd["))
        .unwrap_or(false)
}

/**
 * Check if `log` is an sshd login failure.
 */
fn log_is_sshd_failure(log: &str) -> bool {
    const MSG_START_IDX: usize = 5;
    log_is_sshd(log) && get_field(log, MSG_START_IDX) == Some("Invalid")
}

/**
 * Get all lines in file `path` that are sshd logs with a failed
 * login attempt.
 */
fn get_matching_lines(path: &str) -> io::Result<Vec<String>> {
    let mut result: Vec<String> = Vec::new();

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for next in reader.lines() {
        let line = next?;
        debug!("Checking line: {}", line);

        if log_is_sshd_failure(&line) {
            debug!("Line added");
            result.push(line.clone())
        }
    }

    Ok(result)
}

/**
 * Extract IP address from the sshd log, if present.
 */
fn extract_sshd_ip(log: &str) -> Option<Ipv4Addr> {
    const ADDR_IDX: usize = 9;
    get_field(&log, ADDR_IDX)?
        .parse()
        .ok()
}

/**
 * Macro to generate a function $name that parses a JSON filled with IP range information
 * into a list of IPv4 networks. JSON must have a "prefixes" field containing an array of
 * objects, each with a member $ip_field containing an IPv4 subnet in CIDR notation.
 */
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

/**
 * Check if `addr` is in any of the networks from `nets`.
 */
fn addr_in_networks(addr: &Ipv4Addr, nets: &[Ipv4Net]) -> bool {
    nets.iter().any(|net| net.contains(addr))
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

    let google_nets = parse_google_json(GOOGLE_JSON)?;
    let aws_nets = parse_aws_json(AWS_JSON)?;

    for logfile in files {
        for line in get_matching_lines(&logfile)? {
            debug!("Processing line: {}", &line);

            let addr = extract_sshd_ip(&line).expect("Failed to parse log IP");

            if addr_in_networks(&addr, &google_nets) {
                println!("Google address: {}", addr);
            } else if addr_in_networks(&addr, &aws_nets) {
                println!("AWS address: {}", addr);
            }
        }
    }

    Ok(())
}
