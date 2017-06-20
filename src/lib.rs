extern crate serde;
#[macro_use]
extern crate serde_derive;

// we get an unused import when using macro use, but a
// "this has no effect without macro_use" message otherwise
#[allow(unused_imports)]
#[macro_use]
extern crate snapshot_proc_macro;

pub use snapshot_proc_macro::*;

use std::ffi::OsString;
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub trait Snapable {}
impl<'de, T> Snapable for T where T: Debug + Deserialize<'de> + Serialize {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Snapshot<'a, S: Snapable> {
    pub file: &'a str,
    pub module_path: &'a str,
    pub test_function: &'a str,
    pub recorded_value: S,
}

impl<'a, S> Snapshot<'a, S>
    where S: Snapable
{
    fn path(&self, manifest_dir: &str) -> SnapFileSpec {
        let file_path = &Path::new(self.file);

        let mut components = file_path.components();

        // strip the filename
        let mut filename = components.next_back().unwrap().as_os_str().to_owned();
        filename.push(".snap");

        let mut dir = PathBuf::new();
        for directory in components {
            dir.push(directory.as_os_str());
        }

        dir.push("__snapshots__");

        let mut absolute_path = PathBuf::from(manifest_dir);
        absolute_path.push(dir.clone());
        absolute_path.push(filename.clone());

        let mut relative_path = PathBuf::from(dir.clone());
        relative_path.push(filename.clone());

        SnapFileSpec {
            dir,
            filename,
            absolute_path,
            relative_path,
        }
    }

    pub fn check_snapshot(&self, manifest_dir: &str) {
        let SnapFileSpec {
            dir: snap_dir,
            filename: snap_filename,
            absolute_path,
            relative_path,
        } = self.path(manifest_dir);

        let snap_file = match File::open(&absolute_path) {
            Ok(f) => f,
            Err(why) => {
                panic!("Unable to open snapshot file {:?}: {:?}",
                       relative_path,
                       why.kind())
            }
        };

        unimplemented!();
    }

    pub fn update_snapshot(&self, manifest_dir: &str) {
        let SnapFileSpec {
            dir: snap_dir,
            filename: snap_filename,
            absolute_path,
            relative_path,
        } = self.path(manifest_dir);

        let snap_file = match File::open(&absolute_path) {
            Ok(f) => f,
            Err(why) => {
                panic!("Unable to create snapshot file {:?}: {:?}",
                       relative_path,
                       why.kind())
            }
        };

        unimplemented!();
    }
}

struct SnapFileSpec {
    dir: PathBuf,
    filename: OsString,
    relative_path: PathBuf,
    absolute_path: PathBuf,
}
