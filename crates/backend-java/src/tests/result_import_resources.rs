//! Type/member-only registrations pay the same exact budgets as callable imports.
use super::*;
use crate::{
    ast::{JavaExpr, JavaLiteral},
    dialect::{JavaDependencyScope, JavaResultTypeRole},
    tests::{result_imports::fixture, source_dependency_fixture as source},
};

#[test]
fn result_imports_unused_nominals_and_members_cannot_bypass_limits() {
    let api = fixture::owner();
    let family = api.result_families().next().unwrap();
    let success = family.ty(JavaResultTypeRole::Success);
    let error = family.ty(JavaResultTypeRole::Error);
    let (scope, _) = JavaDependencyScope::new()
        .import_result_type(family.ty(JavaResultTypeRole::Interface))
        .unwrap();
    let (scope, _) = scope
        .import_result_constructor(success.constructor().unwrap())
        .unwrap();
    let (scope, _) = scope
        .import_result_constructor(error.constructor().unwrap())
        .unwrap();
    let (scope, _) = scope
        .import_result_accessor(success.payload_accessor().unwrap())
        .unwrap();
    let bindings = scope.finish();
    let lengths = bindings
        .result_types()
        .map(|value| value.path().encoded_len())
        .chain(
            bindings
                .result_constructors()
                .map(|value| value.owner().path().encoded_len()),
        )
        .chain(
            bindings
                .result_accessors()
                .map(|value| value.path().encoded_len()),
        )
        .collect::<Vec<_>>();
    assert_eq!(lengths.len(), 6);
    let names = lengths.iter().sum();
    let name = *lengths.iter().max().unwrap();
    let package = source::certify(fixture::consumer(
        bindings,
        source::int(),
        vec![],
        JavaExpr::literal(source::int(), JavaLiteral::I32(17)),
    ));
    let exact = || Limits {
        bindings: 6,
        owners: 1,
        names,
        name,
    };
    assert!(check(package.ast(), &exact()).is_empty());
    for fault in 0..4 {
        let mut limits = exact();
        let needle = match fault {
            0 => {
                limits.bindings -= 1;
                "registered dependency bindings"
            }
            1 => {
                limits.owners -= 1;
                "registered dependency owners"
            }
            2 => {
                limits.names -= 1;
                "dependency name bytes"
            }
            3 => {
                limits.name -= 1;
                "dependency qualified-name bytes"
            }
            _ => unreachable!(),
        };
        assert!(
            check(package.ast(), &limits)
                .iter()
                .any(|error| error.message.contains(needle)),
            "{needle}"
        );
    }
}
