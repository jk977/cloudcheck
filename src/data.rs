use std::{
    fs, io, str,
    net::Ipv4Addr,
};

use ipnet::Ipv4Net;

struct HostJson<'a> {
    source_uri: &'a str,
    hostname: &'a str,
    path: &'a str,
    array_pointer: &'a str,
    field: &'a str,
}

impl<'a> HostJson<'a> {
    const fn new(
        source_uri: &'a str,
        hostname: &'a str,
        path: &'a str,
        array_pointer: &'a str,
        field: &'a str
    ) -> Self {
        Self {
            source_uri,
            hostname,
            path,
            array_pointer,
            field
        }
    }
}

struct HostNetwork {
    name: String,
    source_uri: String,
    subnets: Vec<Ipv4Net>,
}

pub struct HostDatabase {
    hosts: Vec<HostNetwork>,
}

impl HostDatabase {
    const DEFAULTS: [HostJson<'static>; 2] = [
        HostJson::new(
            "https://www.gstatic.com/ipranges/cloud.json",
            "Google Cloud",
            "data/google-cloud-ranges.json",
            "/prefixes",
            "ipv4Prefix",
        ),
        HostJson::new(
            "https://ip-ranges.amazonaws.com/ip-ranges.json",
            "Amazon Web Services",
            "data/aws-ranges.json",
            "/prefixes",
            "ip_prefix",
        ),
    ];

    fn from_jsons(host_jsons: &[HostJson]) -> io::Result<Self> {
        let mut result = Self {
            hosts: Vec::with_capacity(host_jsons.len()),
        };

        for json in host_jsons {
            let current = HostNetwork {
                name: json.hostname.to_owned(),
                source_uri: json.source_uri.to_owned(),
                subnets: extract_from_json(json)?,
            };

            result.hosts.push(current);
        }

        Ok(result)
    }

    pub fn with_default_hosts() -> io::Result<Self> {
        Self::from_jsons(&Self::DEFAULTS)
    }

    pub fn get_address_host(&self, addr: Ipv4Addr) -> Option<&str> {
        for host in self.hosts.iter() {
            if host.subnets.iter().find(|subnet| subnet.contains(&addr)).is_some() {
                return Some(&host.name);
            }
        }

        None
    }
}

fn extract_from_json<T: str::FromStr>(host_json: &HostJson) -> io::Result<Vec<T>> {
    macro_rules! make_io_err {
        ($kind:ident, $($err:tt)+) => {
            std::io::Error::new(std::io::ErrorKind::$kind, $($err)+)
        }
    }

    let raw_json = fs::read_to_string(host_json.path)?;
    let json: serde_json::Value = serde_json::from_str(&raw_json)?;
    let prefixes = json
        .pointer(host_json.array_pointer)
        .map(serde_json::Value::as_array)
        .flatten()
        .ok_or_else(|| make_io_err!(InvalidData, "Invalid JSON format"))?;
    let mut subnets: Vec<T> = Vec::new();

    for info in prefixes {
        if let Some(prefix) = info[host_json.field].as_str() {
            let subnet = prefix
                .parse()
                .map_err(|_| make_io_err!(InvalidData, "Invalid Ipv4Net format"))?;
            subnets.push(subnet);
        }
    }

    Ok(subnets)
}
