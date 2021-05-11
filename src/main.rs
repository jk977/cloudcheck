mod data;

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};
use clap::{clap_app, App, AppSettings};
use data::HostDatabase;

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

fn check_address(arg: &str, db: &HostDatabase) -> io::Result<()> {
    let addr = arg
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Ipv4Addr"))?;

    if let Some(hostname) = db.get_address_host(addr) {
        println!("{}: {}", addr, hostname);
    }

    Ok(())
}

fn check_reader_addresses<R: BufRead>(reader: &mut R, db: &HostDatabase) -> io::Result<()> {
    for line in reader.lines() {
        let arg = &line?;
        check_address(arg, &db)?;
    }

    Ok(())
}

fn check_subcmd(matches: &clap::ArgMatches) -> io::Result<()> {
    let db = HostDatabase::with_default_hosts()?;

    if let Some(args) = matches.values_of("ADDRESS") {
        for arg in args {
            check_address(arg, &db)?;
        }
    } else if let Some(paths) = matches.values_of("INPUT_FILE") {
        for path in paths {
            let mut handle = BufReader::new(File::open(path)?);
            check_reader_addresses(&mut handle, &db)?;
        }
    } else {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        check_reader_addresses(&mut handle, &db)?;
    }

    Ok(())
}

fn update_subcmd(_matches: &clap::ArgMatches) -> io::Result<()> {
    unimplemented!()
}

fn main() -> io::Result<()> {
    let matches = build_clap_app().get_matches();

    if let Some(matches) = matches.subcommand_matches("check") {
        check_subcmd(matches)
    } else if let Some(matches) = matches.subcommand_matches("update") {
        update_subcmd(matches)
    } else {
        unreachable!("Failed to cover all subcommands, or Clap is improperly configured")
    }
}
