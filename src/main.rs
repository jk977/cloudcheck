mod data;

use std::io;
use clap::{clap_app, App, AppSettings};
use data::HostDatabase;

macro_rules! die {
    ($($t:tt)+) => {
        eprintln!($($t)+);
        std::process::exit(1);
    }
}

fn build_clap_app() -> App<'static, 'static> {
    clap_app!(cloudcheck =>
        (author: "jk977")
        (about: "Checks IP addresses against ranges owned by various cloud hosts")
        (version: "0.1.0")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (@subcommand update =>
            (about: "Updates the IP ranges used for each host")
        )
        (@subcommand check =>
            (setting: AppSettings::TrailingVarArg)
            (about: "Checks the given IP address(es) against known host ranges")
            (@arg ADDRESSES: +required +multiple "The address(es) to check")
        )
    )
}

fn update_cache() -> io::Result<()> {
    unimplemented!()
}

fn check_addresses<'a, T: Iterator<Item = &'a str>>(args: T) -> io::Result<()> {
    let db = HostDatabase::with_default_hosts()?;

    for arg in args {
        let addr = arg
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Ipv4Addr"))?;
        if let Some(hostname) = db.get_address_host(addr) {
            println!("{}: {}", addr, hostname);
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let matches = build_clap_app().get_matches();

    if matches.subcommand_matches("update").is_some() {
        update_cache()
    } else if let Some(matches) = matches.subcommand_matches("check") {
        if let Some(addresses) = matches.values_of("ADDRESSES") {
            check_addresses(addresses)
        } else {
            die!("Missing IP addresses to check");
        }
    } else {
        unreachable!("Failed to cover all subcommands, or Clap is improperly configured")
    }
}
