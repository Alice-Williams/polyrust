//! Private reconstruction cannot replace structural role or membership checks.
use super::*;
use crate::ast::{
    CContextError, CDeclarationKey, CGeneratedOrigin, CIdentifier, CObjectType, CScalarType,
    CSynthesisReason, numeric_fixture::Fixture,
};

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}

#[test]
fn a_private_owner_wrapper_is_not_registry_membership() {
    let mut f = Fixture::new(&[]);
    let ty = CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::scalar(
        CScalarType::Int,
    ))));
    let local = f
        .registry
        .register_local(&f.scope, key("unbound"), ty)
        .unwrap();
    let forged = COwnerSlotRef { local };
    assert_eq!(
        f.registry.check_owner_slot(&f.function, &forged),
        Err(CRegistryError::UnregisteredReference)
    );
}

#[test]
fn private_inventory_insertion_does_not_bypass_role_shape_revalidation() {
    let mut f = Fixture::new(&[]);
    let local = f.local(CScalarType::Int, "scalar");
    f.registry.owner_slots.insert(COwnerSlotRef {
        local: local.clone(),
    });
    let source = f.source(vec![f.declare(&local, f.int(7))]);
    assert_eq!(
        f.registry.check_package_structure(&[source]),
        Err(CContextError::Registry(CRegistryError::InvalidOwnerSlot))
    );
}
