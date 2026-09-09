//! Private mutation controls ensure requested shapes remain exact registrations.
use super::*;
use crate::ast::{numeric_fixture::Fixture, *};

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}

#[test]
fn shape_count_and_element_tampering_fail_registration_and_expression_construction() {
    let mut f = Fixture::new(&[]);
    let count = f
        .registry
        .register_buffer_count(&f.scope, key("count"))
        .unwrap();
    let other = f
        .registry
        .register_buffer_count(&f.scope, key("other"))
        .unwrap();
    let original = f
        .registry
        .register_buffer_allocation(
            &f.scope,
            key("buffer"),
            CObjectType::scalar(CScalarType::Int),
            count,
            CAllocatorSource::Default,
        )
        .unwrap();
    let mut changed_count = original.clone();
    changed_count.shape = CAllocationShape::Elements(other);
    let mut changed_shape = original.clone();
    changed_shape.shape = CAllocationShape::Object;
    let mut changed_element = original;
    changed_element.object_type = CObjectType::scalar(CScalarType::I64);
    for forged in [changed_count, changed_shape, changed_element] {
        assert_eq!(
            f.registry.check_allocation(&f.scope, &forged),
            Err(CRegistryError::UnregisteredReference)
        );
        let null = f
            .values()
            .literal(CLiteral::NullPointer(
                CNullPointer::new(CObjectType::pointer(CPointerTarget::Void(
                    CConstness::Unqualified,
                )))
                .unwrap(),
            ))
            .unwrap();
        assert!(f.values().allocation_restore(forged, null).is_err());
    }
}
