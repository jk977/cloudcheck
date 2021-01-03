extern crate log;
extern crate env_logger;
extern crate ipnet;
extern crate serde_json;

mod sshd;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::net::Ipv4Addr;

use ipnet::Ipv4Net;
use log::{info, debug};

use sshd::SshdEvent;

const GOOGLE_JSON: &str = "data/google-cloud-ranges.json";
const AWS_JSON: &str = "data/aws-ranges.json";

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
        for event in get_sshd_failures(&logfile)? {
            debug!("Found event: {}", &event.log);

            if addr_in_networks(&event.addr, &google_nets) {
                println!("Google address: {}", &event.addr);
            } else if addr_in_networks(&event.addr, &aws_nets) {
                println!("AWS address: {}", &event.addr);
            }
        }
    }

    Ok(())
}
