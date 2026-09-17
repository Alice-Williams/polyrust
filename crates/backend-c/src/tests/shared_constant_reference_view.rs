//! Read-only source references distinguish expression uses from export-only roots.
use super::{
    c_imported_constants, c_used_imported_constants, constant_consumer_fixture as consumer,
    constant_export_fixture as alias, constant_producer_tests::certify,
    owned_constant_fixture::Shape,
};

#[test]
fn foreign_export_roots_do_not_invent_expression_uses() {
    let producer = consumer::producer(Shape::ConstantsOnly);
    let values: Vec<_> = producer.constants().cloned().collect();
    for fixture in [
        alias::facade(401, &values),
        alias::mixed(402, &values, "own"),
    ] {
        let ready = certify(&fixture);
        assert_eq!(c_imported_constants(&ready).count(), values.len());
        assert_eq!(c_used_imported_constants(&ready).count(), 0);
    }
}

#[test]
fn used_reference_view_retains_original_objects_and_skips_unused_registrations() {
    let producer = consumer::producer(Shape::ConstantsOnly);
    let values: Vec<_> = producer.constants().cloned().collect();
    for usage in [consumer::Usage::Read, consumer::Usage::Unused] {
        let fixture = consumer::fixture(403, &values, None, usage);
        let ready = certify(&fixture);
        assert_eq!(c_imported_constants(&ready).count(), values.len());
        let used: Vec<_> = c_used_imported_constants(&ready).collect();
        assert_eq!(
            used.len(),
            if usage == consumer::Usage::Read {
                values.len()
            } else {
                0
            }
        );
        for value in used {
            assert!(values.contains(value.dependency()));
            assert!(
                c_imported_constants(&ready)
                    .any(|registered| registered.object() == value.object()
                        && registered.dependency() == value.dependency())
            );
        }
    }
}
