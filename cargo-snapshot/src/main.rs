#[macro_use]
extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate snapshot;
extern crate walkdir;

use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;

use clap::{Arg, SubCommand};
use snapshot::SnapFileContents;
use walkdir::WalkDir;

fn main() {
    let input = app_from_crate!()
        .subcommand(
            SubCommand::with_name("update")
                .about("Update snapshots.")
                .arg(Arg::with_name("all").short("a").long("all").help(
                    "Unconditionally update all snapshots, including unrecorded ones.",
                )),
        )
        .get_matches();

    let cwd = ::std::env::current_dir().expect("can't access working directory!");

    // throw away errors
    let existing: Vec<SnapFileContents> = WalkDir::new(cwd)
        .into_iter()
        .map(|r| r.expect("unable to traverse project directory"))
        .filter(|e| e.path().extension() == Some(OsStr::new("snap")))
        .map(|p| BufReader::new(File::open(p.path()).expect("unable to open snapshot file")))
        .map(|r| serde_json::from_reader(r).expect("unable to parse snapshot file"))
        .collect::<Vec<_>>();

    let update_matches = input.subcommand_matches("update");

    if update_matches.is_none() {
        panic!("unsupported command!");
    }

    let mut test_function_names = Vec::new();
    for snap_file in existing {
        for fun in snap_file.keys() {
            test_function_names.push(fun.clone());
        }
    }

    // TODO run tests one-by-one with interactive questions to update individual tests

    // TODO figure out how to handle un-recorded tests
}
