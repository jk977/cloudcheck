use std::fmt;
use std::io;
use std::fs::File;

use ipnet::Ipv4Net;

#[derive(Debug, Clone, Copy)]
pub enum Host {
    GoogleCloud,
    AmazonWebServices,
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::GoogleCloud => "Google Cloud",
            Self::AmazonWebServices => "Amazon Web Services",
        })
    }
}

#[derive(Debug)]
pub struct OwnedNetwork {
    pub host: Host,
    pub subnets: Vec<Ipv4Net>,
}

/**
 * Macro to generate a function $name that parses a JSON filled with IP range information
 * into a list of IPv4 networks. JSON must have a "prefixes" field containing an array of
 * objects, each with a member $ip_field containing an IPv4 subnet in CIDR notation.
 */
macro_rules! gen_json_parser {
    {$name: ident, $host: expr, $ip_field: literal} => {
        pub fn $name(path: &str) -> io::Result<OwnedNetwork> {
            let file = File::open(path)?;
            let json: serde_json::Value = serde_json::from_reader(file)?;
            let prefixes = json["prefixes"]
                .as_array()
                .expect("Unrecognized JSON format");

            let mut subnets: Vec<Ipv4Net> = Vec::new();

            for info in prefixes {
                if let Some(prefix) = info[$ip_field].as_str() {
                    let subnet: Ipv4Net = prefix.parse().expect("Failed to parse IPv4 subnet");
                    subnets.push(subnet);
                }
            }

            Ok(OwnedNetwork { host: $host, subnets })
        }
    }
}

gen_json_parser!{parse_google_data, Host::GoogleCloud, "ipv4Prefix"}
gen_json_parser!{parse_aws_data, Host::AmazonWebServices, "ip_prefix"}
