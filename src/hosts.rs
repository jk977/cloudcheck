use std::io;
use std::fs::File;
use std::net::Ipv4Addr;

use ipnet::Ipv4Net;

#[derive(Debug)]
pub enum Host {
    GoogleCloud,
    AmazonWebServices,
    Unknown,
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

pub fn get_address_host(addr: &Ipv4Addr) -> Host {
    const GOOGLE_JSON: &str = "data/google-cloud-ranges.json";
    const AWS_JSON: &str = "data/aws-ranges.json";

    // TODO: move the JSON parsing to compile time; massive bottleneck otherwise
    let google_nets = parse_google_json(GOOGLE_JSON).expect("Failed to parse Google JSON");
    let aws_nets = parse_aws_json(AWS_JSON).expect("Failed to parse AWS JSON");

    if addr_in_networks(addr, &google_nets) {
        Host::GoogleCloud
    } else if addr_in_networks(addr, &aws_nets) {
        Host::AmazonWebServices
    } else {
        Host::Unknown
    }
}
