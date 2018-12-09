
#[cfg(test)]
mod tests {
    use snapshot::snapshot;

    #[snapshot]
    fn test_if_multiline() -> String {
        "This is a test for \nmultiline strings".to_owned()
    }
}