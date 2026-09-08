//! Interface identities use a real checked CoreIR witness and private C refs.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CConstness, CFunctionType, CObjectType, CRegistryError, CReturnType, CScalarType,
    CWitnessMethod,
};
use portable_core_ir::{CoreDeclaration, CoreProgram};

pub(super) fn core() -> CoreProgram {
    let checked = portable_check::v0::check_program(
        portable_ir::v0::from_json(include_bytes!(
            "../../../build/testdata/registration.poly.json"
        ))
        .unwrap(),
    )
    .unwrap();
    portable_core_ir::lower_checked(&checked).unwrap()
}

#[test]
fn witness_table_adapter_registration_is_one_exact_identity_chain() {
    let core = core();
    let implementation = core
        .module()
        .declarations
        .iter()
        .find_map(|value| match value {
            CoreDeclaration::Implementation(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    let method_id = core.implementations()[implementation.index()].methods[0];
    let method = &core.implementation_methods()[method_id.index()];
    let (mut registry, file) = registry();
    let record = registry.declare_struct(&file, key("Record")).unwrap();
    let interface = registry.declare_struct(&file, key("Interface")).unwrap();
    let table_type = registry.declare_struct(&file, key("Table")).unwrap();
    let signature = CFunctionType::new(CReturnType::Void, vec![]);
    let concrete = registry
        .register_function(&file, key("concrete"), signature.clone())
        .unwrap();
    let adapter_method = registry
        .register_function(&file, key("adapter_method"), signature.clone())
        .unwrap();
    let clone_context = registry
        .register_function(&file, key("clone_context"), signature.clone())
        .unwrap();
    let drop_context = registry
        .register_function(&file, key("drop_context"), signature)
        .unwrap();
    let binding = CWitnessMethod {
        interface_method: method.interface_method,
        implementation_method: method_id,
        concrete,
        adapter: adapter_method,
    };
    assert_eq!(
        registry.register_interface_witness(
            key("DuplicateMethods"),
            implementation,
            &interface,
            &record,
            vec![binding.clone(), binding.clone()]
        ),
        Err(CRegistryError::DuplicateRegistration)
    );
    let witness = registry
        .register_interface_witness(
            key("Witness"),
            implementation,
            &interface,
            &record,
            vec![binding.clone()],
        )
        .unwrap();
    assert_eq!(witness.implementation(), implementation);
    assert_eq!(witness.record(), &record);
    assert_eq!(witness.interface(), &interface);
    assert_eq!(witness.methods(), &[binding]);
    let mutable = registry
        .register_object(
            &file,
            key("MutableTable"),
            CObjectType::structure(table_type.clone()),
        )
        .unwrap();
    let scalar = registry
        .register_object(
            &file,
            key("ScalarTable"),
            CObjectType::scalar(CScalarType::I32)
                .with_constness(CConstness::Const)
                .unwrap(),
        )
        .unwrap();
    for invalid in [&mutable, &scalar] {
        assert_eq!(
            registry.register_interface_table(
                key("TableBinding"),
                &witness,
                invalid,
                &clone_context,
                &drop_context
            ),
            Err(CRegistryError::InterfaceTableType)
        );
    }
    let object = registry
        .register_object(
            &file,
            key("TableObject"),
            CObjectType::structure(table_type)
                .with_constness(CConstness::Const)
                .unwrap(),
        )
        .unwrap();
    let table = registry
        .register_interface_table(
            key("TableBinding"),
            &witness,
            &object,
            &clone_context,
            &drop_context,
        )
        .unwrap();
    assert_eq!(table.witness(), &witness);
    assert_eq!(table.object(), &object);
    assert_eq!(table.clone_context(), &clone_context);
    assert_eq!(table.drop_context(), &drop_context);
    let adapter = registry
        .register_interface_adapter(key("Adapter"), &table)
        .unwrap();
    assert_eq!(adapter.witness(), &witness);
    assert_eq!(adapter.table(), &table);
    registry.check_interface_adapter(&adapter).unwrap();
    assert_eq!(
        registry.register_interface_adapter(key("SecondAdapter"), &table),
        Err(CRegistryError::DuplicateRegistration)
    );
    let (foreign, _) = super::registry_nominals::registry();
    assert_eq!(
        foreign.check_interface_witness(&witness),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        foreign.check_interface_table(&table),
        Err(CRegistryError::CrossRegistry)
    );
    assert_eq!(
        foreign.check_interface_adapter(&adapter),
        Err(CRegistryError::CrossRegistry)
    );
    // No callback ABI or body is certified here. The deliberately simple
    // signatures are registration evidence only; 02D must prove native ABI.
}
