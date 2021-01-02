extern crate env_logger;
extern crate log;
extern crate json;

use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    let _program = args.pop();

    if args.len() != 1 {
        eprintln!("Expected one argument, but got {}", args.len());
        return;
    }

    env_logger::init();
}
