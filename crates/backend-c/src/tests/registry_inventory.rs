//! Canonical snapshots and contract references remain independent of spellings.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CAggregateRef, CCallableContractOrigin, CFunctionType, CMemberBinding, CObjectType,
    CRegistrationKind, CRegistryError, CReturnType, CScalarType,
};

#[test]
fn canonical_nominal_function_inventory_ignores_registration_order_and_brand() {
    let (mut a, a_file) = registry();
    let (mut b, b_file) = registry();
    for (registry, file, names) in [
        (&mut a, &a_file, ["Alpha", "Zeta"]),
        (&mut b, &b_file, ["Zeta", "Alpha"]),
    ] {
        for name in names {
            let record = registry.declare_struct(file, key(name)).unwrap();
            let owner = CAggregateRef::Struct(record.clone());
            let field = registry
                .register_member(&owner, key("value"), CObjectType::scalar(CScalarType::I32))
                .unwrap();
            registry.define_aggregate(&owner, vec![field]).unwrap();
            registry
                .register_typedef(
                    file,
                    key(&format!("{name}Alias")),
                    CObjectType::structure(record),
                )
                .unwrap();
            let function = registry
                .register_function(
                    file,
                    key(&format!("{name}Function")),
                    CFunctionType::new(CReturnType::Void, vec![]),
                )
                .unwrap();
            let scope = registry
                .register_scope(&function, None, key("root"))
                .unwrap();
            registry
                .register_local(&scope, key("local"), CObjectType::scalar(CScalarType::I32))
                .unwrap();
        }
    }
    let before = a.inventory();
    assert_eq!(before, b.inventory());
    assert_eq!(format!("{before:?}"), format!("{:?}", b.inventory()));
    assert_eq!(before.len(), 12);
    let frozen = a.freeze();
    assert_eq!(frozen.registrations().inventory(), before);
    let clone = frozen.clone();
    assert_eq!(clone, frozen);
    assert_eq!(
        clone.registrations().inventory(),
        b.freeze().registrations().inventory()
    );
}

#[test]
fn callable_member_requires_the_exact_contract_not_just_the_same_signature() {
    let (mut registry, file) = registry();
    let signature = CFunctionType::new(CReturnType::Void, vec![]);
    let first = registry
        .register_function(&file, key("first"), signature.clone())
        .unwrap();
    let second = registry
        .register_function(&file, key("second"), signature)
        .unwrap();
    assert_eq!(first.signature(), second.signature());
    assert_ne!(first.contract(), second.contract());
    assert_eq!(
        first.contract().origin(),
        CCallableContractOrigin::GeneratedBody
    );
    assert_eq!(first.contract().file(), &file);
    registry.check_callable_contract(first.contract()).unwrap();
    let table = CAggregateRef::Struct(registry.declare_struct(&file, key("Table")).unwrap());
    let member = registry
        .register_callable_member(&table, key("invoke"), &first)
        .unwrap();
    assert_eq!(
        member.binding(),
        &CMemberBinding::Callable(first.contract().clone())
    );
    registry
        .check_callable_member(&table, &member, &first)
        .unwrap();
    assert_eq!(
        registry.check_callable_member(&table, &member, &second),
        Err(CRegistryError::CallableContractMismatch)
    );
    let unbound = registry
        .register_member(&table, key("unbound"), member.ty().clone())
        .unwrap();
    assert_eq!(
        registry.check_callable_member(&table, &unbound, &first),
        Err(CRegistryError::CallableContractMismatch)
    );
    let (foreign, _) = super::registry_nominals::registry();
    assert_eq!(
        foreign.check_callable_contract(first.contract()),
        Err(CRegistryError::CrossRegistry)
    );
    assert!(
        registry
            .inventory()
            .iter()
            .any(|value| value.kind == CRegistrationKind::Member)
    );
}
