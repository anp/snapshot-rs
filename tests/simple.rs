#![feature(proc_macro)]

extern crate snapshot;

mod test {
    use snapshot::snapshot;

    #[snapshot]
    fn simple_snapshot() -> i32 {
        let x = 1;
        x
    }
}
