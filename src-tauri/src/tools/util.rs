pub fn nullable(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}

pub fn timestamp_dir_now() -> String {
    chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_dir_has_expected_shape() {
        let value = timestamp_dir_now();
        assert_eq!(value.len(), 15);
        assert_eq!(value.as_bytes()[8], b'-');
        assert!(value
            .chars()
            .enumerate()
            .all(|(idx, ch)| idx == 8 || ch.is_ascii_digit()));
    }

    #[test]
    fn nullable_trims_and_filters_empty() {
        assert_eq!(nullable(Some("  hello  ")), Some("hello"));
        assert_eq!(nullable(Some("   ")), None);
        assert_eq!(nullable(None), None);
    }
}
