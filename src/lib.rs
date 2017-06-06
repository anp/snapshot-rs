extern crate serde;

#[macro_use]
extern crate snapshot_proc_macro;

pub use snapshot_proc_macro::*;

use std::fmt::Debug;
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
