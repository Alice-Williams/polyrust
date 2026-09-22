//! Resolved-owner accounting and transactional resource failures.
use super::*;
use crate::{ast::*, dialect::JavaQualifiedName, tests::infinite_constants::expression};
use portable_binary64::Binary64Sign;

#[test]
fn infinity_bound_charges_short_and_qualified_owners_and_rejects_missing_names() {
    for (name, width) in [
        (
            JavaResolvedName::Local(JavaIdentifier::new("Double").unwrap()),
            "Double".len(),
        ),
        (
            JavaResolvedName::Qualified(JavaQualifiedName::Type(JavaKnownType::Double)),
            "java.lang.Double".len(),
        ),
    ] {
        for (sign, member) in [
            (Binary64Sign::Positive, "POSITIVE_INFINITY"),
            (Binary64Sign::Negative, "NEGATIVE_INFINITY"),
        ] {
            let names = BTreeMap::from([(
                TargetSymbolRef::KnownType(JavaKnownType::Double),
                name.clone(),
            )]);
            let mut reader = Reader {
                budget: Budget::new(),
                names: &names,
            };
            reader.expression(&expression(sign), 0).unwrap();
            assert_eq!(
                reader.budget.bytes(),
                256 + (width + 1 + member.len()) as u64
            );
            let mut reader = Reader {
                budget: Budget::new(),
                names: &names,
            };
            reader.budget.add(256 * 1024 * 1024 - 256).unwrap();
            assert!(reader.expression(&expression(sign), 0).is_err());
        }
    }
    let names = BTreeMap::new();
    let mut reader = Reader {
        budget: Budget::new(),
        names: &names,
    };
    assert!(
        reader
            .expression(&expression(Binary64Sign::Positive), 0)
            .is_err()
    );
    assert!(
        reader
            .expression(&expression(Binary64Sign::Negative), 257)
            .is_err()
    );
}
