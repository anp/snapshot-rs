#![feature(proc_macro)]

#[macro_use]
extern crate snapshot;

mod test {
    #[snapshot]
    fn simple_snapshot() {
        let x = 1;
        snap!(x);
    }
}
