use serde_derive::{Deserialize, Serialize};

use fs2::FileExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use pretty_assertions::assert_eq;

static OS_LOCK_FILE_FAIL: &str = "Your OS failed to lock the '.snap' file!";
static OS_CLONE_FILE_FAIL: &str = "Your OS Failed to clone file handle";

pub type SnapFileContents = BTreeMap<String, Snapshot<serde_json::Value>>;

pub trait Snapable {}
impl<T> Snapable for T where T: Debug + DeserializeOwned + Serialize {}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Snapshot<S: Snapable> {
    pub file: Vec<String>,
    pub module_path: String,
    pub test_function: String,
    pub recorded_value: S,
}

impl<S> Snapshot<S>
where
    S: Snapable + Debug + DeserializeOwned + PartialEq + Serialize,
{
    pub fn new(
        file: String,
        module_path: String,
        test_function: String,
        recorded_value: S,
    ) -> Self {
        Snapshot {
            file: Path::new(&file)
                .components()
                .map(|component| component.as_os_str().to_str().unwrap().to_owned())
                .collect(),
            module_path,
            test_function,
            recorded_value,
        }
    }

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

        let mut module_snapshots = parse_snaps_from_file(&snap_file, &relative_path);

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

        let mut file = match OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&absolute_path)
        {
            Ok(f) => f,
            Err(why) => panic!(
                "Unable to open or create snapshot file {:?}: {:?}",
                relative_path,
                why.kind()
            ),
        };

        file.lock_exclusive().expect(OS_LOCK_FILE_FAIL);

        let mut existing_snaps: SnapFileContents = parse_snaps_from_file(&file, &relative_path);

        // Now we need to update the particular snapshot we care about
        existing_snaps.insert(self.module_key(), self.create_deserializable());

        write_snaps_to_file(&mut file, &existing_snaps, &relative_path);

        // We don't care if unlock fails because the OS will automatically unlock the file
        //  when it closes or the process terminates.  We will be closing the file handle
        //  on drop.
        match file.unlock() {
            _ => (),
        };
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
        let mut components = self.file.iter();

        // strip the filename
        let mut filename = components.next_back().unwrap().clone();
        filename.push_str(".snap");

        let mut dir = PathBuf::new();
        for directory in components {
            dir.push(directory);
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

fn truncate_file(file: &mut File) {
    let file_len = file.seek(SeekFrom::Current(0)).unwrap();
    file.set_len(file_len).unwrap();
}

fn parse_snaps_from_file(file: &File, relative_path: &Path) -> SnapFileContents {
    let mut contents = String::new();
    let mut reader = BufReader::new(file.duplicate().expect(OS_CLONE_FILE_FAIL));
    reader.read_to_string(&mut contents).unwrap();

    match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(why) => {
            if contents.len() == 0 {
                eprintln!(
                    "Snapshot file does not appear to exist: {:?}",
                    relative_path
                );
                SnapFileContents::new()
            } else {
                eprintln!(
                    "Unable to parse potentially corrupt snapshot file {:?}: {:?}",
                    relative_path, why
                );

                SnapFileContents::new()
            }
        }
    }
}

fn write_snaps_to_file(file: &mut File, snapshots: &SnapFileContents, relative_path: &Path) {
    file.seek(SeekFrom::Start(0)).unwrap();

    let writer = BufWriter::new(file.duplicate().expect(OS_CLONE_FILE_FAIL));
    match serde_json::to_writer_pretty(writer, &snapshots) {
        Err(why) => panic!(
            "Unable to serialize or write snapshot result to {:?}: {:?}",
            relative_path, why
        ),
        _ => {}
    }

    truncate_file(file);
}

struct SnapFileSpec {
    dir: PathBuf,
    relative_path: PathBuf,
    absolute_path: PathBuf,
}
