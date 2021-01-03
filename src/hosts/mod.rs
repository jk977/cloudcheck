mod data;

use std::net::Ipv4Addr;
use ipnet::Ipv4Net;

pub use data::{parse_google_data, parse_aws_data, Host, OwnedNetwork};

/**
 * Check if `addr` is in any of the networks from `nets`.
 */
fn addr_in_nets(addr: &Ipv4Addr, nets: &[Ipv4Net]) -> bool {
    nets.iter().any(|net| net.contains(addr))
}

/**
 * Search `nets` for an IPv4 subnet that contains `addr` and return the host, if any.
 */
pub fn get_address_host(addr: &Ipv4Addr, nets: &[OwnedNetwork]) -> Option<Host> {
    nets
        .iter()
        .find(|net| addr_in_nets(addr, &net.subnets))
        .map(|net| net.host)
}
