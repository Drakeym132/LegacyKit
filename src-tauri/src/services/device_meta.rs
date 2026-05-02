/// Maps an Apple device's marketing product type (e.g. "iPhone6,1") to its
/// processor generation (A4 = 4, A6 = 6, A10 = 10).
///
/// Mirrors the JS table in `src/lib/utils/processorGen.ts`. Returns `None` for
/// product types outside the legacy range this app targets.
pub fn infer_processor_gen(product_type: &str) -> Option<u8> {
    let p = product_type;

    if p.starts_with("iPhone1,") || p.starts_with("iPhone2,") || p.starts_with("iPod1,") || p.starts_with("iPod2,") {
        return Some(1);
    }
    if p == "iPod3,1" {
        return Some(3);
    }
    if p.starts_with("iPhone3,") || p == "iPad1,1" || p == "iPod4,1" {
        return Some(4);
    }
    if p == "iPhone4,1" || p.starts_with("iPad2,") || p == "iPod5,1" {
        return Some(5);
    }
    if p.starts_with("iPad3,") {
        // iPad3,1..3 → A5X (5), iPad3,4..6 → A6X (6)
        if let Some(suffix) = p.strip_prefix("iPad3,") {
            if let Ok(n) = suffix.parse::<u8>() {
                if (1..=3).contains(&n) {
                    return Some(5);
                }
                if (4..=6).contains(&n) {
                    return Some(6);
                }
            }
        }
    }
    if p.starts_with("iPhone5,") {
        return Some(6);
    }
    if p.starts_with("iPhone6,") || p.starts_with("iPad4,") {
        return Some(7);
    }
    if p.starts_with("iPhone7,") || p == "iPod7,1" || p.starts_with("iPad5,") {
        return Some(8);
    }
    if p.starts_with("iPhone8,") || p.starts_with("iPad6,") {
        return Some(9);
    }
    if p.starts_with("iPhone9,") || p.starts_with("iPad7,") {
        return Some(10);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iphone5_1_is_a6() {
        assert_eq!(infer_processor_gen("iPhone5,1"), Some(6));
    }

    #[test]
    fn ipad3_split_a5x_a6x() {
        assert_eq!(infer_processor_gen("iPad3,1"), Some(5));
        assert_eq!(infer_processor_gen("iPad3,3"), Some(5));
        assert_eq!(infer_processor_gen("iPad3,4"), Some(6));
        assert_eq!(infer_processor_gen("iPad3,6"), Some(6));
    }

    #[test]
    fn a7_through_a10() {
        assert_eq!(infer_processor_gen("iPhone6,1"), Some(7));
        assert_eq!(infer_processor_gen("iPhone7,2"), Some(8));
        assert_eq!(infer_processor_gen("iPhone8,1"), Some(9));
        assert_eq!(infer_processor_gen("iPhone9,3"), Some(10));
    }

    #[test]
    fn unknown_returns_none() {
        assert_eq!(infer_processor_gen("iPhone20,1"), None);
        assert_eq!(infer_processor_gen(""), None);
    }
}
