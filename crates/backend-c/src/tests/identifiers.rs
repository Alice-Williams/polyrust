//! Independent C17 lexical admission matrix.

use crate::ast::{CIdentifier, CKeyword, CNameError, CReservedMacro};
use std::collections::BTreeSet;

#[test]
fn c17_keywords_are_exhaustive_unique_and_protected() {
    let expected: BTreeSet<_> = "
        _Alignas _Alignof _Atomic _Bool _Complex _Generic _Imaginary _Noreturn
        _Static_assert _Thread_local auto break case char const continue
        default do double else enum extern float for goto if inline int long
        register restrict return short signed sizeof static struct switch
        typedef union unsigned void volatile while
    "
    .split_whitespace()
    .collect();
    let actual: BTreeSet<_> = CKeyword::ALL.into_iter().map(CKeyword::as_str).collect();
    assert_eq!(expected.len(), 44);
    assert_eq!(actual, expected);
    for word in expected {
        let keyword = CKeyword::from_spelling(word).unwrap();
        assert_eq!(CIdentifier::new(word), Err(CNameError::Keyword(keyword)));
    }
}

#[test]
fn ordinary_names_are_preserved_without_target_source_injection() {
    for name in [
        "value",
        "value_2",
        "Class",
        "class",
        "poly_result",
        "while_loop",
    ] {
        assert_eq!(CIdentifier::new(name).unwrap().as_str(), name);
        assert_eq!(CKeyword::from_spelling(name), None);
    }
}

#[test]
fn malformed_names_report_typed_errors_and_byte_positions() {
    for (name, expected) in [
        ("", CNameError::Empty),
        ("9value", CNameError::InvalidStart),
        ("é", CNameError::InvalidStart),
        ("a b", CNameError::InvalidContinuation { byte_offset: 1 }),
        ("a-b", CNameError::InvalidContinuation { byte_offset: 1 }),
        ("a\n", CNameError::InvalidContinuation { byte_offset: 1 }),
        ("a\0", CNameError::InvalidContinuation { byte_offset: 1 }),
        ("aé", CNameError::InvalidContinuation { byte_offset: 1 }),
        ("a/**/b", CNameError::InvalidContinuation { byte_offset: 1 }),
    ] {
        assert_eq!(CIdentifier::new(name), Err(expected), "{name:?}");
    }
}

#[test]
fn implementation_names_and_standard_macros_cannot_be_user_symbols() {
    for name in ["_value", "__value", "_Value", "_"] {
        assert_eq!(
            CIdentifier::new(name),
            Err(CNameError::ImplementationReserved)
        );
    }
    for (word, kind) in [
        ("bool", CReservedMacro::Bool),
        ("true", CReservedMacro::True),
        ("false", CReservedMacro::False),
        ("NULL", CReservedMacro::Null),
    ] {
        assert_eq!(CIdentifier::new(word), Err(CNameError::StandardMacro(kind)));
        assert_eq!(kind.as_str(), word);
    }
}

#[test]
fn spelling_wrappers_have_value_identity_without_mutable_access() {
    let a = CIdentifier::new("poly_value").unwrap();
    let b = CIdentifier::new("poly_value").unwrap();
    assert_eq!(a, b);
    assert_eq!(BTreeSet::from([a, b]).len(), 1);
    assert!(!CNameError::Keyword(CKeyword::While).to_string().is_empty());
}
