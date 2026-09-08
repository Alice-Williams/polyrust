//! Conversion registrations are retained; runtime provenance proof is separate.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CAllocatorSource, CConstness as Q, CConversion, CExpressionError as E, CExpressions,
    CFunctionType, CInterfaceAdapterRef, CLiteral, CNullPointer, CObjectType, CPointerTarget,
    CRegistry, CRegistryError, CReturnType, CScalarType, CValueKind, CWitnessMethod,
};
use portable_core_ir::CoreDeclaration;

fn pointer(ty: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
}
fn void(qualifier: Q) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Void(qualifier))
}
fn null(ast: &CExpressions<'_>, ty: CObjectType) -> crate::ast::CValue {
    ast.literal(CLiteral::NullPointer(CNullPointer::new(ty).unwrap()))
        .unwrap()
}

fn adapter_fixture() -> (CRegistry, CInterfaceAdapterRef) {
    let core = super::registry_interfaces::core();
    let implementation = core
        .module()
        .declarations
        .iter()
        .find_map(|value| match value {
            CoreDeclaration::Implementation(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    let (mut registry, file) = registry();
    let record = registry.declare_struct(&file, key("Record")).unwrap();
    let interface = registry.declare_struct(&file, key("Interface")).unwrap();
    let table = registry.declare_struct(&file, key("Table")).unwrap();
    let function = registry
        .register_function(
            &file,
            key("callback"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let methods = core.implementations()[implementation.index()]
        .methods
        .iter()
        .map(|id| CWitnessMethod {
            interface_method: core.implementation_methods()[id.index()].interface_method,
            implementation_method: *id,
            concrete: function.clone(),
            adapter: function.clone(),
        })
        .collect();
    let witness = registry
        .register_interface_witness(key("Witness"), implementation, &interface, &record, methods)
        .unwrap();
    let object = registry
        .register_object(
            &file,
            key("table"),
            CObjectType::structure(table)
                .with_constness(Q::Const)
                .unwrap(),
        )
        .unwrap();
    let table = registry
        .register_interface_table(key("TableBinding"), &witness, &object, &function, &function)
        .unwrap();
    let adapter = registry
        .register_interface_adapter(key("Adapter"), &table)
        .unwrap();
    // Registration fixture only, deliberately not a valid callback ABI/body.
    (registry, adapter)
}

#[test]
fn adapter_conversions_derive_exact_record_and_preserve_qualifiers() {
    let (registry, adapter) = adapter_fixture();
    let ast = CExpressions::new(&registry);
    for qualifier in [Q::Unqualified, Q::Const] {
        let record = CObjectType::structure(adapter.witness().record().clone())
            .with_constness(qualifier)
            .unwrap();
        let operand = null(&ast, pointer(record.clone()));
        let erased = ast.adapter_erase(adapter.clone(), operand.clone()).unwrap();
        assert_eq!(
            erased.kind(),
            &CValueKind::Convert {
                conversion: CConversion::AdapterErase(Box::new(adapter.clone())),
                operand: Box::new(operand),
            }
        );
        assert_eq!(erased.ty(), &void(qualifier));
        let restored = ast
            .adapter_restore(adapter.clone(), erased.clone())
            .unwrap();
        assert_eq!(
            restored.kind(),
            &CValueKind::Convert {
                conversion: CConversion::AdapterRestore(Box::new(adapter.clone())),
                operand: Box::new(erased),
            }
        );
        assert_eq!(restored.ty(), &pointer(record));
    }
    let other = CObjectType::structure(adapter.witness().interface().clone());
    assert_eq!(
        ast.adapter_erase(adapter.clone(), null(&ast, pointer(other.clone()))),
        Err(E::InvalidPointerConversion)
    );
    assert_eq!(
        ast.adapter_restore(adapter.clone(), null(&ast, pointer(other))),
        Err(E::InvalidPointerConversion)
    );
    let (foreign, _) = super::registry_nominals::registry();
    let foreign = CExpressions::new(&foreign);
    assert_eq!(
        foreign.adapter_restore(adapter, null(&foreign, void(Q::Unqualified))),
        Err(E::Registry(CRegistryError::CrossRegistry))
    );
}

#[test]
fn allocation_restore_derives_target_from_registration_without_inventing_live_state() {
    let (mut registry, file) = registry();
    let function = registry
        .register_function(
            &file,
            key("run"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("body"))
        .unwrap();
    let target = CObjectType::scalar(CScalarType::I64);
    let allocation = registry
        .register_allocation(
            &scope,
            key("allocation"),
            target.clone(),
            CAllocatorSource::Default,
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let operand = null(&ast, void(Q::Unqualified));
    let value = ast
        .allocation_restore(allocation.clone(), operand.clone())
        .unwrap();
    assert_eq!(
        value.kind(),
        &CValueKind::Convert {
            conversion: CConversion::AllocationRestore(Box::new(allocation.clone())),
            operand: Box::new(operand),
        }
    );
    assert_eq!(value.ty(), &pointer(target.clone()));
    assert_eq!(
        ast.allocation_restore(allocation.clone(), null(&ast, void(Q::Const))),
        Err(E::InvalidPointerConversion)
    );
    assert_eq!(
        ast.allocation_restore(allocation.clone(), null(&ast, pointer(target))),
        Err(E::InvalidPointerConversion)
    );
    let (foreign, _) = super::registry_nominals::registry();
    let foreign = CExpressions::new(&foreign);
    assert_eq!(
        foreign.allocation_restore(allocation, null(&foreign, void(Q::Unqualified))),
        Err(E::Registry(CRegistryError::CrossRegistry))
    );
}
