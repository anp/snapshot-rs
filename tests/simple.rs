#![feature(proc_macro)]

#[macro_use]
extern crate snapshot;
use snapshot::snapshot;

mod test {
    #[snapshot]
    fn simple_snapshot() -> i32 {
        let x = 1;
        x
    }
}
