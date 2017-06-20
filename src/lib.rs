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
    pub fn path(&self, crate_root: &str) -> (PathBuf, OsString) {
        let file_path = &Path::new(self.file);

        let mut components = file_path.components();

        // strip the filename
        let mut file_to_write = components.next_back().unwrap().as_os_str().to_owned();
        file_to_write.push(".snap");

        let mut ret = PathBuf::from(crate_root);
        for dir in components {
            ret.push(dir.as_os_str());
        }

        ret.push("__snapshots__");

        (ret, file_to_write)
    }

    pub fn check_snapshot(&self, manifest_dir: &str) {
        unimplemented!();
    }

    pub fn update_snapshot(&self, manifest_dir: &str) {
        unimplemented!();
    }
}
