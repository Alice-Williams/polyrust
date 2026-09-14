use super::{
    budget::{Budget, MAX_BYTES},
    bundle::reconcile,
    json::{Encoder, Reservation, Sink},
};
use std::collections::BTreeSet;

#[test]
fn byte_boundaries_are_checked_and_transactional() {
    let mut budget = Budget::default();
    budget.add(MAX_BYTES).unwrap();
    assert!(budget.add(1).is_err());
    assert_eq!(budget.0, MAX_BYTES);
    assert!(budget.add(u64::MAX).is_err());
    assert_eq!(budget.0, MAX_BYTES);
}

#[test]
fn strings_escape_control_unicode_and_syntax_within_reservation() {
    for value in ["", "plain", "\"\\\n\t\0\u{1f}", "🦀 café <>& /", "\\u0022"] {
        let mut reservation = Reservation::default();
        reservation.string(value).unwrap();
        let mut encoder = Encoder::new(reservation.0.0);
        encoder.string(value).unwrap();
        let encoded = encoder.finish();
        assert!(encoded.len() as u64 <= reservation.0.0);
        assert!(!encoded.chars().any(|c| c < ' '));
        let mut short = Encoder::new(encoded.len() as u64 - 1);
        assert!(short.string(value).is_err());
        let mut exact = Encoder::new(encoded.len() as u64);
        exact.string(value).unwrap();
        assert_eq!(exact.finish(), encoded);
    }
    let mut encoder = Encoder::new(100);
    encoder.string("\"\\\n\0").unwrap();
    assert_eq!(encoder.finish(), "\"\\\"\\\\\\u000a\\u0000\"");
}

#[test]
fn reconciliation_rejects_missing_replaced_duplicate_and_oversize_payloads() {
    let names = BTreeSet::from(["a.java".into(), "a.api.json".into(), "bundle.json".into()]);
    let files: Vec<_> = names
        .iter()
        .map(|name: &String| (name.clone(), "x".into()))
        .collect();
    reconcile(&files, &names, 3).unwrap();
    assert!(reconcile(&files, &names, 2).is_err());
    assert!(reconcile(&files[..2], &names, 3).is_err());
    let mut duplicate = files.clone();
    duplicate.push(files[0].clone());
    assert!(reconcile(&duplicate, &names, 4).is_err());
    let mut replaced = files;
    replaced[0].0 = "wrong.json".into();
    assert!(reconcile(&replaced, &names, 3).is_err());
}
