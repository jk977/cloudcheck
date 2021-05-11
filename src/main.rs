mod data;

use std::io;
use clap::{clap_app, App, AppSettings};
use data::HostDatabase;
use std::io::BufRead;

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
            (about:
                concat!(
                    "Checks the given IPv4 address(es) against known host ranges. If none are ",
                    "provided, addresses are read from standard input.",
                )
            )
            (@arg INPUT_FILE:
                -f --files +takes_value +multiple
                "Files to check, with one IP address per line"
            )
            (@arg ADDRESS:
                -a --addresses +takes_value +multiple
                "IP addresses to check"
            )
        )
    )
}

fn update_cache() -> io::Result<()> {
    unimplemented!()
}

fn check_address(arg: &str, db: &HostDatabase) -> io::Result<()> {
    let addr = arg
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Ipv4Addr"))?;

    if let Some(hostname) = db.get_address_host(addr) {
        println!("{}: {}", addr, hostname);
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let matches = build_clap_app().get_matches();

    if matches.subcommand_matches("update").is_some() {
        update_cache()
    } else if let Some(matches) = matches.subcommand_matches("check") {
        let db = HostDatabase::with_default_hosts()?;

        if let Some(args) = matches.values_of("ADDRESS") {
            for arg in args {
                check_address(arg, &db)?;
            }

            Ok(())
        } else if let Some(files) = matches.values_of("INPUT_FILE") {
            unimplemented!()
        } else {
            let stdin = io::stdin();
            let mut handle = stdin.lock();

            for line in handle.lines() {
                let arg = &line?;
                check_address(arg, &db)?;
            }

            Ok(())
        }
    } else {
        unreachable!("Failed to cover all subcommands, or Clap is improperly configured")
    }
}
