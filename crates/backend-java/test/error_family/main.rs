//! Six-state target family only; no compiler source admission or owner imports.
mod fixture;
mod model;
mod native;
mod rejection;

use fixture::*;
use portable_backend_java::dialect::*;
use portable_codegen::*;

#[test]
fn six_original_error_roles_are_distinct_and_old_payload_free_profile_stays_closed() {
    let family = checked(Fault::None);
    assert_eq!(family.payload_name().as_str(), "value");
    let values = RustIntegerErrorKind::ALL.map(|kind| family.error_value(kind));
    assert_eq!(
        values
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        6
    );
    assert!(
        JavaScalarResultFamily::from_certificate(family.package().clone(), family.types()).is_err()
    );
}

#[test]
fn local_six_state_family_has_strict_native_value_and_mutation_proof() {
    native::prove();
}
