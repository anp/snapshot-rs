#[macro_use]
extern crate pretty_assertions;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

// we get an unused import when using macro use, but a
// "this has no effect without macro_use" message otherwise
#[allow(unused_imports)]
#[macro_use]
extern crate snapshot_proc_macro;

pub use snapshot_proc_macro::*;

use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fs::{create_dir_all, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait Snapable {}
impl<T> Snapable for T where T: Debug + DeserializeOwned + Serialize {}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Snapshot<S: Snapable> {
    pub file: String,
    pub module_path: String,
    pub test_function: String,
    pub recorded_value: S,
}

impl<S> Snapshot<S>
    where S: Snapable + Debug + DeserializeOwned + PartialEq + Serialize
{
    pub fn check_snapshot(&self, manifest_dir: &str) {
        let SnapFileSpec {
            absolute_path,
            relative_path,
            ..
        } = self.path(manifest_dir);

        let snap_file = match File::open(&absolute_path) {
            Ok(f) => f,
            Err(why) => {
                panic!("Unable to open snapshot file {:?}: {:?}",
                       relative_path,
                       why.kind())
            }
        };

        let rdr = BufReader::new(snap_file);
        let module_snapshots: HashMap<String, Snapshot<S>> = match serde_json::from_reader(rdr) {
            Ok(ps) => ps,
            Err(why) => {
                panic!("Unable to parse previous snapshot as the correct type:\n{:#?}",
                       why);
            }
        };

        let snap_key = self.module_key();
        let previous_snapshot = match module_snapshots.get(&snap_key) {
            Some(s) => s,
            None => {
                panic!("Unable to find snapshot for test {:?} in {:?}",
                       snap_key,
                       relative_path)
            }
        };

        assert_eq!(self.file,
                   previous_snapshot.file,
                   "Filename for snapshot test function doesn't match recorded one");

        assert_eq!(self.module_path,
                   previous_snapshot.module_path,
                   "Module paths for snapshot test function doesn't match recorded one");

        assert_eq!(self.test_function,
                   previous_snapshot.test_function,
                   "Test function name doesn't match recorded one");

        assert_eq!(self.recorded_value,
                   previous_snapshot.recorded_value,
                   "Test output doesn't match recorded snapshot!");

        // just as a catch all in case we need other fields?
        assert_eq!(self, previous_snapshot, "Snapshot metadata is corrupt!");
    }

    pub fn update_snapshot(&self, manifest_dir: &str) {
        let SnapFileSpec {
            dir: snap_dir,
            filename: snap_filename,
            absolute_path,
            relative_path,
        } = self.path(manifest_dir);

        let mut dir_to_create = PathBuf::from(manifest_dir);
        dir_to_create.push(snap_dir.clone());
        match create_dir_all(&dir_to_create) {
            Ok(_) => (),
            Err(why) => {
                panic!("Unable to create snapshots directory {:?}: {:?}",
                       snap_dir,
                       why.kind())
            }
        }

        let snap_file = match File::create(&absolute_path) {
            Ok(f) => f,
            Err(why) => {
                panic!("Unable to create snapshot file {:?}: {:?}",
                       relative_path,
                       why.kind())
            }
        };

        // TODO write snapshot to disk

        unimplemented!();
    }

    fn module_key(&self) -> String {
        let mut snapshot_key = self.module_path.to_owned();
        snapshot_key.push_str("::");
        snapshot_key.push_str(&self.test_function);
        snapshot_key
    }

    fn path(&self, manifest_dir: &str) -> SnapFileSpec {
        let file_path = &Path::new(&self.file);

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
}

struct SnapFileSpec {
    dir: PathBuf,
    filename: OsString,
    relative_path: PathBuf,
    absolute_path: PathBuf,
}
