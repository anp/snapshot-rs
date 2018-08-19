extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate snapshot;

mod test {
    use snapshot::snapshot;

    #[snapshot]
    fn simple_snapshot() -> i32 {
        let x = 1;
        x
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Lol {
        a: f32,
        x: i32,
        z: String,
    }

    #[snapshot]
    fn compound_snapshot() -> Lol {
        Lol {
            a: 1.0,
            x: 12,
            z: String::from("woowwowow"),
        }
    }
}
