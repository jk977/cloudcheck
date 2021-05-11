use std::{fs, io, net::Ipv4Addr, str};

use ipnet::Ipv4Net;

pub struct HostJson<'a> {
    hostname: &'a str,
    path: &'a str,
    array_pointer: &'a str,
    field: &'a str,
}

impl<'a> HostJson<'a> {
    pub const fn new(
        hostname: &'a str,
        path: &'a str,
        array_pointer: &'a str,
        field: &'a str,
    ) -> Self {
        Self {
            hostname,
            path,
            array_pointer,
            field,
        }
    }
}

struct HostNetwork {
    name: String,
    subnets: Vec<Ipv4Net>,
}

pub struct HostDatabase {
    hosts: Vec<HostNetwork>,
}

impl HostDatabase {
    const DEFAULTS: [HostJson<'static>; 2] = [
        HostJson::new(
            "Google Cloud",
            "data/google-cloud-ranges.json",
            "/prefixes",
            "ipv4Prefix",
        ),
        HostJson::new(
            "Amazon Web Services",
            "data/aws-ranges.json",
            "/prefixes",
            "ip_prefix",
        ),
    ];

    pub fn from_jsons(host_jsons: &[HostJson]) -> io::Result<Self> {
        let mut result = Self {
            hosts: Vec::with_capacity(host_jsons.len()),
        };

        for json in host_jsons {
            let current = HostNetwork {
                name: json.hostname.to_owned(),
                subnets: extract_from_json(json)?,
            };

            result.hosts.push(current);
        }

        Ok(result)
    }

    pub fn with_default_hosts() -> io::Result<Self> {
        Self::from_jsons(&Self::DEFAULTS)
    }

    pub fn get_host(&self, addr: Ipv4Addr) -> Option<&str> {
        for host in self.hosts.iter() {
            let has_addr = host
                .subnets
                .iter()
                .find(|subnet| subnet.contains(&addr))
                .is_some();

            if has_addr {
                return Some(&host.name);
            }
        }

        None
    }
}

fn extract_from_json<T: str::FromStr>(host_json: &HostJson) -> io::Result<Vec<T>> {
    let raw_json = fs::read_to_string(host_json.path)?;
    let json: serde_json::Value = serde_json::from_str(&raw_json)?;
    let prefixes = json
        .pointer(host_json.array_pointer)
        .map(serde_json::Value::as_array)
        .flatten()
        .ok_or_else(|| make_io_err!(InvalidData, "Invalid JSON format: {}", host_json.path))?;
    let mut subnets: Vec<T> = Vec::new();

    for info in prefixes {
        if let Some(prefix) = info[host_json.field].as_str() {
            let subnet = prefix
                .parse()
                .map_err(|_| make_io_err!(InvalidData, "Invalid Ipv4Net format: {}", prefix))?;
            subnets.push(subnet);
        }
    }

    Ok(subnets)
}
