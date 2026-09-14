#![forbid(unsafe_code)]
#[path = "../src/output/bundle_budget.rs"]
mod budget;

#[test]
fn exact_count_and_aggregate_byte_boundaries() {
    for count in [0, 1025, usize::MAX] {
        assert!(budget::BundleBudget::new(count).is_err());
    }
    for count in [1, 1024] {
        let mut value = budget::BundleBudget::new(count).unwrap();
        let initial = value.bytes();
        value.include(256 * 1024 * 1024 - initial - 1, 1).unwrap();
        assert_eq!(value.bytes(), 256 * 1024 * 1024);
        assert!(value.include(0, 1).is_err());
        assert!(value.include(1, 0).is_err());
        assert!(value.include(u64::MAX, 0).is_err());
        assert!(value.include(0, u64::MAX).is_err());
        assert_eq!(
            value.bytes(),
            256 * 1024 * 1024,
            "failed additions are transactional"
        );
    }
}
