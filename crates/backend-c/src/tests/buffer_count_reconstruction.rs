//! Count aliases are checked structurally; a forged typed local is not a binding.
use super::*;
use crate::ast::{numeric_fixture::Fixture, *};

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}

#[test]
fn aliases_keep_const_size_requirements_and_changed_local_metadata_is_rejected() {
    for constness in [CConstness::Const, CConstness::Unqualified] {
        let mut f = Fixture::new(&[]);
        let alias = f
            .registry
            .register_typedef(
                &f.file,
                key("Count"),
                CObjectType::scalar(CScalarType::Size)
                    .with_constness(constness)
                    .unwrap(),
            )
            .unwrap();
        let local = f
            .registry
            .register_local(&f.scope, key("count"), CObjectType::typedef(alias))
            .unwrap();
        let result = f.registry.buffer_count_reference(&local);
        if constness == CConstness::Const {
            assert_eq!(result.unwrap().local(), &local);
        } else {
            assert_eq!(result, Err(E::InvalidBufferCount));
        }
        let mut forged = local;
        forged.ty = CObjectType::scalar(CScalarType::Size)
            .with_constness(CConstness::Const)
            .unwrap();
        assert_eq!(
            f.registry.buffer_count_reference(&forged),
            Err(E::UnregisteredReference)
        );
    }
}

#[test]
fn a_private_wrapper_cannot_turn_mutable_storage_into_a_buffer_count() {
    let mut f = Fixture::new(&[]);
    let mutable = f.local(CScalarType::Size, "mutable");
    let forged = CBufferCountRef {
        local: Arc::new(mutable),
    };
    assert_eq!(
        f.registry.register_buffer_allocation(
            &f.scope,
            key("buffer"),
            CObjectType::scalar(CScalarType::Int),
            forged,
            CAllocatorSource::Default,
        ),
        Err(E::InvalidBufferCount)
    );
}
