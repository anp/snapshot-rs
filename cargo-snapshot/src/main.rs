#[macro_use]
extern crate clap;

use clap::{Arg, SubCommand};

fn main() {
    let input = app_from_crate!()
        .subcommand(
            SubCommand::with_name("update")
                .about("Update snapshots.")
                .arg(Arg::with_name("all").short("a").help(
                    "Unconditionally update all snapshots, including unrecorded ones.",
                )),
        )
        .get_matches();
    println!("Hello, world!");
}
