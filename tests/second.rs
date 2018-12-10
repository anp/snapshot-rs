mod test {
    use snapshot::snapshot;

    #[snapshot]
    fn another_simple_snapshot() -> i32 {
        let x = 2;
        x
    }
}
