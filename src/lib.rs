extern crate serde;

#[macro_use]
extern crate snapshot_proc_macro;

pub use snapshot_proc_macro::*;

use std::ffi::OsString;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub trait Snapable {
    fn check_snapshot(&self, md: Metadata) -> Result<(), ()>;
    fn write_snapshot(&self, md: Metadata) -> Result<(), ()>;
    fn update_snapshot(&self, md: Metadata) -> Result<(), ()>;
}

impl<'de, T> Snapable for T
    where T: Debug + Deserialize<'de> + Serialize
{
    fn check_snapshot(&self, md: Metadata) -> Result<(), ()> {
        unimplemented!();
    }

    fn write_snapshot(&self, md: Metadata) -> Result<(), ()> {
        unimplemented!();
    }

    fn update_snapshot(&self, md: Metadata) -> Result<(), ()> {
        unimplemented!();
    }
}

#[derive(Debug)]
pub struct Metadata<'a> {
    pub test_function: &'a str,
    pub file: &'a str,
    pub module_path: &'a str,
}

impl<'a> Metadata<'a> {
    pub fn path(&self, crate_root: &str) -> (PathBuf, OsString) {
        let mut ret = PathBuf::from(crate_root);

        let file_path = &Path::new(self.file);

        let mut components = file_path.components();

        // strip the filename
        let file_to_write = components.next_back().unwrap().as_os_str().to_owned();

        for dir in components {
            ret.push(dir.as_os_str());
        }

        ret.push("__snapshots__");

        // FIXME replace the .rs extension with something sane
        (ret, file_to_write)
    }
}
