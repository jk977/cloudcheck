#[macro_use]
mod util;
mod data;

use clap::clap_app;
use data::HostDatabase;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

fn build_clap_app() -> clap::App<'static, 'static> {
    clap_app!(cloudcheck =>
        (author: "jk977")
        (about:
            concat!(
                "Checks the given IPv4 address(es) against known host ranges. If none are ",
                "provided, addresses are read from standard input.",
            )
        )
        (version: "0.1.0")
        (@group input =>
            (@arg INPUT_FILE:
                -f --files +takes_value +multiple
                "Files to check, with one IP address per line"
            )
            (@arg ADDRESS:
                -a --addresses +takes_value +multiple
                "IP addresses to check"
            )
        )
        (@arg CSV:
            -c --csv +required +takes_value
            "String specifying a path to a CSV with columns \"HOSTNAME,PATH,POINTER,FIELD\" where HOSTNAME is the name of the host (e.g., \"Google Cloud\"), PATH is the path to the JSON file containing the IP ranges, POINTER is a JSON pointer to an array of objects in the CSV file, and FIELD is the object field that contains the IP address."
        )
    )
}

fn check_address(arg: &str, db: &HostDatabase) -> io::Result<()> {
    let addr = arg
        .parse()
        .map_err(|_| make_io_err!(InvalidData, "Invalid Ipv4Addr: {}", arg))?;

    if let Some(hostname) = db.get_host(addr) {
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

fn main() -> io::Result<()> {
    let matches = build_clap_app().get_matches();
    let csv = matches.value_of("CSV").map(Path::new).unwrap();
    let db = HostDatabase::from_hosts_csv(csv)?;

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
