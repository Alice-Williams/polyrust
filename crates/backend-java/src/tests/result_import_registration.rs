//! The mutable consuming builder rejects capacity failures before returning handles.
use super::*;
use crate::{dialect::JavaResultTypeRole, tests::result_imports::fixture};

#[test]
fn result_import_registration_exact_limits_and_duplicates() {
    let api = fixture::owner();
    let original = api
        .result_families()
        .next()
        .unwrap()
        .ty(JavaResultTypeRole::Success);
    let name = original.path().encoded_len();
    let (scope, value) = JavaDependencyScope::new()
        .import_result_type(original.clone())
        .unwrap();
    assert_eq!(scope.name_bytes, name);
    let (scope, _) = scope.import_result_type(original.clone()).unwrap();
    assert_eq!(scope.name_bytes, name);
    let (scope, _) = scope
        .import_result_constructor(original.constructor().unwrap())
        .unwrap();
    assert_eq!(scope.name_bytes, 2 * name);
    let (scope, _) = scope
        .import_result_accessor(original.payload_accessor().unwrap())
        .unwrap();
    assert_eq!(
        scope.name_bytes,
        2 * name + value.path().encoded_len() + ".value".len()
    );
    for fault in 0..5 {
        let (mut scope, _) = JavaDependencyScope::new()
            .import_result_type(original.clone())
            .unwrap();
        // Exercise the same registration checker with exact reduced capacities.
        scope.owners.clear();
        scope.name_bytes = 0;
        let mut limits = Limits {
            bindings: 1,
            owners: 1,
            names: name,
            name,
        };
        match fault {
            0 => {}
            1 => limits.bindings -= 1,
            2 => limits.owners -= 1,
            3 => limits.names -= 1,
            4 => limits.name -= 1,
            _ => unreachable!(),
        }
        let result =
            scope.verify_registration_with_limits(api.package_identity(), Some(name), &limits);
        assert_eq!(result.is_ok(), fault == 0, "fault {fault}: {result:?}");
    }
    // A public consuming operation fails rather than returning a partially updated scope.
    let mut full = JavaDependencyScope::new();
    full.name_bytes = LIMITS.names;
    assert!(full.import_result_type(original).is_err());
}
