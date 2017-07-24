#[macro_use]
extern crate clap;
extern crate dialoguer;
#[macro_use]
extern crate error_chain;
extern crate serde;
extern crate serde_json;
extern crate snapshot;
extern crate walkdir;

use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::process::{Command, Stdio};

use clap::{Arg, SubCommand};
use dialoguer::{Checkboxes, Select};
use snapshot::SnapFileContents;
use walkdir::WalkDir;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, SnapResult;
    }

    foreign_links {
        Io(::std::io::Error);
    }
}

quick_main!(run);

fn run() -> SnapResult<()> {
    let input = app_from_crate!()
        .subcommand(
            SubCommand::with_name("update")
                .about("Update snapshots.")
                .arg(Arg::with_name("all").short("a").long("all").help(
                    "Unconditionally update all snapshots, including unrecorded ones.",
                )),
        )
        .get_matches();

    let update_matches = input.subcommand_matches("update");

    if let Some(update_matches) = update_matches {
        if update_matches.is_present("all") {
            let status = Command::new("cargo")
                .arg("test")
                .env("UPDATE_SNAPSHOTS", "1")
                .stdout(Stdio::null()) // FIXME(dikaiosune) capture w JSON output
                .stderr(Stdio::null()) // FIXME(dikaiosune) and print nice message
                .status()
                .chain_err(|| "unable to execute cargo")?;

            if !status.success() {
                bail!("unable to update all snapshots!");
            }
        } else {
            interactive_process()?;
        }
        println!("\nAll updates processed!");
    } else {
        panic!("unsupported command");
    }

    Ok(())
}

fn interactive_process() -> SnapResult<()> {
    let cwd = ::std::env::current_dir()
        .chain_err(|| "unable to read cwd")?;

    // FIXME(adam) don't just throw away errors;
    let existing: Vec<SnapFileContents> = WalkDir::new(cwd)
        .into_iter()
        .map(|r| r.expect("unable to traverse project directory"))
        .filter(|e| e.path().extension() == Some(OsStr::new("snap")))
        .map(|p| BufReader::new(File::open(p.path()).expect("unable to open snapshot file")))
        .map(|r| serde_json::from_reader(r).expect("unable to parse snapshot file"))
        .collect::<Vec<_>>();

    let mut test_function_names = Vec::new();
    for snap_file in existing {
        for fun in snap_file.keys() {
            test_function_names.push(fun.clone());
        }
    }

    let mut failed_tests = Vec::new();

    println!("Checking for out of date snapshot tests...");
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
            .stdout(Stdio::null()) // FIXME(dikaiosune) these should be captured as machine readable
            .stderr(Stdio::null()) // FIXME(dikaiosune) and just the tes
            .status()
            .chain_err(|| "unable to execute cargo")?;

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

            let mut run_test = true;
            while run_test {
                let run_status = Command::new("cargo")
                    .arg("test")
                    .arg(fn_to_update)
                    .env("UPDATE_SNAPSHOTS", "1")
                    .stdout(Stdio::null()) // FIXME(dikaiosune) these should be captured as JSON
                    .stderr(Stdio::null()) // FIXME(dikaiosune) and just the test output shown
                    .status()
                    .chain_err(|| "unable to execute cargo")?;

                if run_status.success() {
                    run_test = false;
                } else {
                    println!("\nUpdating {} failed! What would you like to do?",
                             fn_to_update);

                    let options = ["Retry", "Skip", "Abort"];

                    let selection = Select::new()
                        .items(&options)
                        .interact()
                        .chain_err(|| "unable to retrieve user input")?;

                    if options[selection] == "Retry" {
                        continue;
                    } else if options[selection] == "Skip" {
                        break;
                    } else if options[selection] == "Abort" {
                        ::std::process::exit(1);
                    } else {
                        bail!("invalid menu selection")
                    }
                }
            }
        }

        Ok(())
    }
}
