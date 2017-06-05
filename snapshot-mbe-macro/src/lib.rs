use std::convert::AsRef;
use std::fmt::Debug;
use std::path::Path;

#[macro_export]
macro_rules! test_metadata {
    ($e:expr) => {{
        let mut file = ::std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        file.push(file!());

        let line = line!();
        let column = column!();
        let module_path = module_path!();
        let snapshot_name = stringify!($e);
        let test_function = ::snapshot::__test_function_name(&file, line, column);

        ::snapshot::Metadata {
            file, line, column, module_path, name: snapshot_name, test_function,
        }
    }};
}

#[doc(hidden)]
pub fn __test_function_name<P>(path: P, line: u32, column: u32) -> String
    where P: AsRef<Path> + Debug
{
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::open(path).unwrap();

    let mut src = String::new();
    file.read_to_string(&mut src);

    //let foo = syn::parsing::parse_crate(&src);

    //panic!("{:#?}", foo);

    src
}

#[macro_export]
macro_rules! snap {
    ($e:expr) => {
        let metadata = test_metadata!($e);

        let update_snapshots = match ::std::env::var("UPDATE_SNAPSHOTS") {
            Ok(_) => true,
            Err(_) => false,
        };
        panic!("\n{:#?}\n", metadata);
        $e
    };
}
