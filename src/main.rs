#[macro_use]
mod util;
mod data;

use clap::clap_app;
use data::{HostDatabase, HostJson};
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    mem::MaybeUninit,
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
        (@arg JSON:
            -j --("use-jsons") +takes_value +multiple
            "String specifying a host JSON, formatted \"HOSTNAME,PATH,POINTER,FIELD\" where HOSTNAME is the name of the host (e.g., \"Google Cloud\"), PATH is the path to the JSON file, POINTER is a pointer to an array of objects, and FIELD is object field that contains the IP address."
        )
    )
}

fn parse_user_jsons(vals: clap::Values) -> io::Result<Vec<HostJson>> {
    let mut host_jsons = Vec::with_capacity(vals.len());

    for val in vals {
        const REQUIRED_FIELDS: usize = 4;
        let mut fields = MaybeUninit::<[&str; REQUIRED_FIELDS]>::uninit();
        let field_count = val.matches(',').count() + 1;

        if field_count != REQUIRED_FIELDS {
            let e = make_io_err!(
                InvalidData,
                "Invalid number of fields in argument \"{}\" (expected {}, got {})",
                val,
                REQUIRED_FIELDS,
                field_count
            );
            return Err(e);
        }

        for (i, field) in val.split(',').enumerate() {
            unsafe {
                (fields.as_mut_ptr() as *mut &str).add(i).write(field);
            }
        }

        let [hostname, path, pointer, field] = unsafe { fields.assume_init() };
        host_jsons.push(HostJson::new(hostname, path, pointer, field));
    }

    Ok(host_jsons)
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
    let db = if let Some(vals) = matches.values_of("JSON") {
        let host_jsons = parse_user_jsons(vals)?;
        HostDatabase::from_jsons(&host_jsons)?
    } else {
        HostDatabase::with_default_hosts()?
    };

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
