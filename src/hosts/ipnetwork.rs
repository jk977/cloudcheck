use ipnet::Ipv4Net;
use std::io::{self, Read};

trait IpNetwork {
    fn parse_ipv4_ranges<R: Read>(reader: R) -> io::Result<Vec<Ipv4Net>>;
}
