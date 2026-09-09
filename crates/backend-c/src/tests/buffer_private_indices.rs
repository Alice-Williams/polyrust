//! Private selection identities cannot fabricate precision from a range or type.
use super::*;
use crate::ast::{CScalarType, numeric_fixture::Fixture};

#[test]
fn selected_bindings_join_ranges_but_unrelated_bindings_do_not_join() {
    let mut f = Fixture::new(&[]);
    let local = f.local(CScalarType::Size, "index");
    let other = f.local(CScalarType::Size, "other");
    let source = Key::local(&local);
    let other = Key::local(&other);
    let singleton = ElementIndex::checked(0, 0, Some(&source)).unwrap();
    let wider = ElementIndex::checked(0, 7, Some(&source)).unwrap();
    let different = ElementIndex::checked(0, 7, Some(&other)).unwrap();
    assert_eq!(singleton.join_selection(&wider), Some(wider.clone()));
    assert_eq!(wider.join_selection(&singleton), Some(wider.clone()));
    assert_eq!(wider.join_selection(&different), None);
    assert!(singleton.touches(|root| root == source.root()));
    assert!(wider.exact());
    assert_eq!(wider.constant_value(), None);
    assert!(!ElementIndex::constant(0).touches(|_| true));
}

#[test]
fn malformed_ranges_and_wrong_scalar_categories_cannot_claim_a_selected_binding() {
    let mut f = Fixture::new(&[]);
    assert_eq!(ElementIndex::checked(2, 1, None), Err(E::IndexOutOfBounds));
    for ty in [
        CScalarType::Bool,
        CScalarType::Int,
        CScalarType::I64,
        CScalarType::F64,
    ] {
        let local = f.local(ty, &format!("index_{ty:?}"));
        let index = ElementIndex::checked(0, 7, Some(&Key::local(&local))).unwrap();
        assert!(!index.exact());
        assert!(!index.touches(|_| true));
    }
    let interval = ElementIndex::checked(0, 7, None).unwrap();
    assert!(!interval.exact());
    assert_eq!(interval.join_selection(&interval), None);
}

#[test]
fn singleton_observations_never_erase_their_current_binding_at_a_join() {
    let mut f = Fixture::new(&[]);
    let left = Key::local(&f.local(CScalarType::Size, "left"));
    let right = Key::local(&f.local(CScalarType::Size, "right"));
    let left = ElementIndex::checked(0, 0, Some(&left)).unwrap();
    let right = ElementIndex::checked(0, 0, Some(&right)).unwrap();
    let literal = ElementIndex::constant(0);
    assert_eq!(left.join_selection(&left), Some(left.clone()));
    assert_eq!(literal.join_selection(&literal), Some(literal.clone()));
    for (left, right) in [(&left, &right), (&left, &literal), (&literal, &right)] {
        assert_eq!(left.join_selection(right), None);
        assert_eq!(right.join_selection(left), None);
    }
}
