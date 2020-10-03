/* SPDX-License-Identifier: GPL-2.0 */

use getopts::Options;
use std::env;

pub struct Opts {
    pub interface: String,
}

pub fn parse() -> Option<Opts> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();

    opts.optmulti("i", "interface", "the interface to listen on", "INTERFACE");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{}\n", f);
            print_usage(&program, opts);
            return None;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return None;
    }
    let interface = matches.opt_str("i");
    if interface.is_none() {
        print_usage(&program, opts);
        return None;
    };

    Some(Opts {
        interface: interface.unwrap(),
    })
}

// private functions

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}
