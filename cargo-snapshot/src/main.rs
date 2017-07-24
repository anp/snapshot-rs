#[macro_use]
extern crate clap;
extern crate dialoguer;
extern crate serde;
extern crate serde_json;
extern crate snapshot;
extern crate walkdir;

use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::process::{Command, Stdio};

use clap::{Arg, SubCommand};
use dialoguer::Checkboxes;
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

    let mut failed_tests = Vec::new();

    println!("Checking for out of date snapshot tests...\n");
    for test_fn in test_function_names {
        let mut test_fn_chunks = test_fn.splitn(2, "::");

        // skip the crate name
        test_fn_chunks
            .next()
            .expect("looks like an empty test function name");

        let real_test_fn = test_fn_chunks
            .next()
            .expect("seemingly malformed test name");

        let first_run_status = Command::new("cargo")
            .arg("test")
            .arg(&real_test_fn)
// uncomment these once we can get just the error message back from cargo/rust
//            .stdout(Stdio::null())
//            .stderr(Stdio::null())
            .status()
            .expect("unable to execute cargo");

        if !first_run_status.success() {
            failed_tests.push(real_test_fn.to_owned());
        }
    }

    if failed_tests.is_empty() {
        println!("\nNo snapshot tests require an update!");
        ::std::process::exit(0);
    } else {
        println!("\nPlease select which snapshot tests should be updated:");
        println!("  (press <Space> to select, <Enter> to submit)\n");

        let mut menu = Checkboxes::new();
        for failed in &failed_tests {
            menu.item(failed);
        }
        let all_to_update = menu.interact().expect("error accepting user selections");

        for fn_idx in all_to_update {
            let fn_to_update = &failed_tests[fn_idx];
            println!("Updating {}...", &fn_to_update);

            // TODO actually run the tests
        }

        println!("\nAll updates processed!");
    }

    // TODO figure out how to handle un-recorded tests
}
