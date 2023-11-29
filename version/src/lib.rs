pub mod version;

#[cfg(test)]
mod tests {
    use crate::version::*;
    #[test]
    fn test_empty_version() {
        set_version("".into()).unwrap();
        let output = build_version().unwrap();
        assert_eq!(&*output, "dev");
    }

    #[test]
    fn test_not_empty_version() {
        set_version("test-version".into()).unwrap();
        let output = build_version().unwrap();
        assert_eq!(&*output, "test-version");
    }
}
