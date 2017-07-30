#[macro_use]
extern crate clap;
extern crate dialoguer;
extern crate duct;
#[macro_use]
extern crate error_chain;
extern crate serde;
extern crate serde_json;
extern crate snapshot;
extern crate walkdir;

use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;

use clap::{Arg, SubCommand};
use dialoguer::{Checkboxes, Select};
use duct::cmd;
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

    if let Some(update_matches) = input.subcommand_matches("update") {
        if update_matches.is_present("all") {
            let output = cmd("cargo", &["test"])
                .env("UPDATE_SNAPSHOTS", "1")
                .stdout_capture()
                .stderr_capture()
                .run()
                .chain_err(|| "unable to execute cargo")?;

            if !output.status.success() {
                // TODO(dikaiosune) print what failed
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
    let test_function_names = find_existing_snapshot_test_names()?;
    let mut failed_tests = Vec::new();

    println!("Checking for out of date snapshot tests...");
    for test_fn in &test_function_names {
        if !test_fn.success()? {
            failed_tests.push(test_fn);
        }
    }

    if failed_tests.is_empty() {
        println!("\nNo snapshot tests require an update!");
        ::std::process::exit(0);
    } else {
        println!("\nPlease select which snapshot tests should be updated:");
        println!("  <Space> to select, <Enter> to submit\n");

        let mut menu = Checkboxes::new();
        for failed in &failed_tests {
            menu.item(&failed.0);
        }

        let all_to_update = menu.interact()
            .chain_err(|| "error accepting user selections")?;

        for fn_idx in all_to_update {
            let fn_to_update = &failed_tests[fn_idx];
            println!("Updating {}...", &fn_to_update.0);

            // attempt to update the snapshot in a loop, allowing user to change code in bt runs
            let mut run_test = true;
            while run_test {
                let run_output = cmd("cargo", &["test", &fn_to_update.0])
                    .env("UPDATE_SNAPSHOTS", "1")
                    .stdout_capture()
                    .stderr_capture()
                    .run()
                    .chain_err(|| "unable to execute cargo")?;

                if run_output.status.success() {
                    run_test = false;
                } else {
                    println!("\nUpdating {} failed! What would you like to do?",
                             &fn_to_update.0);

                    match capture_failure_selection()? {
                        TestFailureSelection::Retry => continue,
                        TestFailureSelection::Skip => break,
                        TestFailureSelection::Abort => {
                            println!("\nExiting...");
                            ::std::process::exit(1);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

struct FnName(String);

impl FnName {
    // TODO this should let the user see why the test failed, or warn them if it wasn't from
    // snapshot failure
    fn success(&self) -> SnapResult<bool> {
        Ok(cmd("cargo", &["test", &self.0])
               .run()
               .chain_err(|| "unable to execute cargo")?
               .status
               .success())
    }
}

fn find_existing_snapshot_test_names() -> SnapResult<Vec<FnName>> {
    let cwd = ::std::env::current_dir()
        .chain_err(|| "unable to read cwd")?;

    let mut existing: Vec<SnapFileContents> = Vec::new();

    let snap_extension = OsStr::new("snap");
    for walk_result in WalkDir::new(cwd) {
        let entry = walk_result
            .chain_err(|| "unable to traverse project directory")?;

        if entry.path().extension() == Some(snap_extension) {
            let rdr = BufReader::new(File::open(entry.path())
                                         .chain_err(|| "unable to open snapshot file")?);
            let contents = serde_json::from_reader(rdr)
                .chain_err(|| "unable to parse snapshot file")?;
            existing.push(contents);
        }
    }

    let mut test_function_names = Vec::new();
    for snap_file in existing {
        for fun in snap_file.keys() {
            // now we need to strip crate name from the string
            let mut test_fn_chunks = fun.splitn(2, "::");

            match test_fn_chunks.next() {
                Some(_) => (),
                None => bail!("malformed test fn name: empty"),
            }

            let real_test_fn = match test_fn_chunks.next() {
                Some(n) => n,
                None => bail!("malformed test fn name: only had crate name"),
            };

            test_function_names.push(FnName(real_test_fn.to_owned()));
        }
    }
    Ok(test_function_names)
}

enum TestFailureSelection {
    Retry,
    Skip,
    Abort,
}

fn capture_failure_selection() -> SnapResult<TestFailureSelection> {
    let options = ["Retry", "Skip", "Abort"];

    let selection = Select::new()
        .items(&options)
        .interact()
        .chain_err(|| "unable to retrieve user input")?;

    if options[selection] == "Retry" {
        Ok(TestFailureSelection::Retry)
    } else if options[selection] == "Skip" {
        Ok(TestFailureSelection::Skip)
    } else if options[selection] == "Abort" {
        Ok(TestFailureSelection::Abort)
    } else {
        bail!("invalid menu selection")
    }
}
