use serde_derive::{Deserialize, Serialize};

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub type SnapFileContents = BTreeMap<String, Snapshot<serde_json::Value>>;

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
where
    S: Snapable + Debug + DeserializeOwned + PartialEq + Serialize,
{
    pub fn check_snapshot(&self, manifest_dir: &str) {
        let SnapFileSpec {
            absolute_path,
            relative_path,
            ..
        } = self.path(manifest_dir);

        let snap_file = match File::open(&absolute_path) {
            Ok(f) => f,
            Err(why) => panic!(
                "Unable to open snapshot file {:?}: {:?}",
                relative_path,
                why.kind()
            ),
        };

        let rdr = BufReader::new(snap_file);
        let mut module_snapshots: SnapFileContents = match serde_json::from_reader(rdr) {
            Ok(ps) => ps,
            Err(why) => {
                panic!("Unable to parse previous snapshot:\n{:#?}", why);
            }
        };

        let snap_key = self.module_key();
        let previous_snapshot = match module_snapshots.remove(&snap_key) {
            Some(s) => s,
            None => panic!(
                "Unable to find snapshot for test {:?} in {:?}",
                snap_key, relative_path
            ),
        };

        let Snapshot {
            recorded_value,
            file,
            module_path,
            test_function,
        } = previous_snapshot;

        match serde_json::from_value(recorded_value) {
            Ok(recorded_value) => {
                assert_eq!(
                    self.file, file,
                    "Filename for snapshot test function doesn't match recorded one"
                );

                assert_eq!(
                    self.module_path, module_path,
                    "Module paths for snapshot test function doesn't match recorded one"
                );

                assert_eq!(
                    self.test_function, test_function,
                    "Test function name doesn't match recorded one"
                );

                assert_eq!(
                    self.recorded_value, recorded_value,
                    "Test output doesn't match recorded snapshot!"
                );
            }
            Err(why) => panic!(
                "Unable to parse existing snapshot as correct type: {:?}",
                why
            ),
        }
    }

    pub fn update_snapshot(&self, manifest_dir: &str) {
        let SnapFileSpec {
            dir: snap_dir,
            absolute_path,
            relative_path,
            ..
        } = self.path(manifest_dir);

        let mut dir_to_create = PathBuf::from(manifest_dir);
        dir_to_create.push(snap_dir.clone());
        match create_dir_all(&dir_to_create) {
            Ok(_) => (),
            Err(why) => panic!(
                "Unable to create snapshots directory {:?}: {:?}",
                snap_dir,
                why.kind()
            ),
        }

        let mut existing_snaps: SnapFileContents = match File::open(&absolute_path) {
            Ok(f) => {
                let mut rdr = BufReader::new(f);
                let mut contents = String::new();
                rdr.read_to_string(&mut contents)
                    .expect("Unable to read snapshot file we just opened.");

                match serde_json::from_str(&contents) {
                    Ok(v) => v,
                    Err(why) => panic!(
                        "Unable to parse potentially corrupt snapshot file {:?}: {:?}",
                        relative_path, why
                    ),
                }
            }
            Err(why) => match why.kind() {
                ::std::io::ErrorKind::NotFound => SnapFileContents::new(),
                _ => panic!(
                    "Unable to open existing snapshot file {:?}: {:?}",
                    relative_path,
                    why.kind()
                ),
            },
        };

        // now we need to update the particular snapshot we care about
        existing_snaps.insert(self.module_key(), self.create_deserializable());

        let writer = BufWriter::new(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&absolute_path)
                .expect("Unable to open/create file that we just opened/created!"),
        );

        match serde_json::to_writer_pretty(writer, &existing_snaps) {
            Ok(_) => (),
            Err(why) => panic!(
                "Unable to serialize or write snapshot result to {:?}: {:?}",
                relative_path, why
            ),
        }
    }

    fn module_key(&self) -> String {
        let mut snapshot_key = self.module_path.to_owned();
        snapshot_key.push_str("::");
        snapshot_key.push_str(&self.test_function);
        snapshot_key
    }

    fn create_deserializable(&self) -> Snapshot<serde_json::Value> {
        match serde_json::to_value(&self.recorded_value) {
            Ok(v) => Snapshot {
                file: self.file.clone(),
                test_function: self.test_function.clone(),
                module_path: self.module_path.clone(),
                recorded_value: v,
            },
            Err(why) => panic!("Unable to serialize test value: {:?}", why),
        }
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
            absolute_path,
            relative_path,
        }
    }
}

struct SnapFileSpec {
    dir: PathBuf,
    relative_path: PathBuf,
    absolute_path: PathBuf,
}
