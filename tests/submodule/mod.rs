
#[cfg(test)]
mod tests {
    use snapshot::snapshot;

    #[snapshot]
    fn test_if_two() -> u32 {
        2
    }
}