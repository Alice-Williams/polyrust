//! Direct dependency imports cannot acquire unimplemented callback roles.
use super::*;
use portable_core_ir::CoreDeclaration;

#[test]
fn imported_callables_cannot_be_addresses_indirect_contracts_or_callable_members() {
    let (api, _) = dependency(CScalarType::I32);
    let mut registry = CRegistry::new();
    let file = file(&mut registry);
    let imported = registry
        .import_function(api.functions().next().unwrap().clone())
        .unwrap();
    let owned = registry
        .register_function(&file, key("local"), imported.signature().clone())
        .unwrap();
    let record = registry.declare_struct(&file, key("record")).unwrap();
    let aggregate = CAggregateRef::Struct(record);
    assert!(
        registry
            .register_callable_member(&aggregate, key("imported"), &imported)
            .is_err()
    );
    let member = registry
        .register_callable_member(&aggregate, key("local"), &owned)
        .unwrap();
    assert!(
        registry
            .check_callable_member(&aggregate, &member, &imported)
            .is_err()
    );
    assert!(
        registry
            .check_callable_member(&aggregate, &member, &owned)
            .is_ok()
    );
    let expressions = CExpressions::new(&registry);
    assert!(expressions.function_address(imported.clone()).is_err());
    let pointer = expressions.function_address(owned.clone()).unwrap();
    assert!(
        expressions
            .indirect(pointer.clone(), imported.clone())
            .is_err()
    );
    assert!(expressions.indirect(pointer, owned).is_ok());
    assert!(expressions.direct(imported).is_ok());
}

#[test]
fn every_interface_callback_role_requires_an_owned_function() {
    let checked = portable_check::v0::check_program(
        portable_ir::v0::from_json(include_bytes!(
            "../../../build/testdata/registration.poly.json"
        ))
        .unwrap(),
    )
    .unwrap();
    let core = portable_core_ir::lower_checked(&checked).unwrap();
    let implementation = core
        .module()
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            CoreDeclaration::Implementation(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    let method_id = core.implementations()[implementation.index()].methods[0];
    let method = &core.implementation_methods()[method_id.index()];
    let (api, _) = dependency(CScalarType::I32);
    let mut registry = CRegistry::new();
    let file = file(&mut registry);
    let imported = registry
        .import_function(api.functions().next().unwrap().clone())
        .unwrap();
    let owned = registry
        .register_function(&file, key("owned"), imported.signature().clone())
        .unwrap();
    let record = registry.declare_struct(&file, key("record")).unwrap();
    let interface = registry.declare_struct(&file, key("interface")).unwrap();
    for (concrete, adapter) in [(&imported, &owned), (&owned, &imported)] {
        assert!(
            registry
                .register_interface_witness(
                    key("witness"),
                    implementation,
                    &interface,
                    &record,
                    vec![CWitnessMethod {
                        interface_method: method.interface_method,
                        implementation_method: method_id,
                        concrete: concrete.clone(),
                        adapter: adapter.clone(),
                    }]
                )
                .is_err()
        );
    }
    let witness = registry
        .register_interface_witness(
            key("witness"),
            implementation,
            &interface,
            &record,
            vec![CWitnessMethod {
                interface_method: method.interface_method,
                implementation_method: method_id,
                concrete: owned.clone(),
                adapter: owned.clone(),
            }],
        )
        .unwrap();
    let object = registry
        .register_object(
            &file,
            key("table"),
            CObjectType::structure(interface)
                .with_constness(CConstness::Const)
                .unwrap(),
        )
        .unwrap();
    for (clone_context, drop_context) in [(&imported, &owned), (&owned, &imported)] {
        assert!(
            registry
                .register_interface_table(
                    key("table_binding"),
                    &witness,
                    &object,
                    clone_context,
                    drop_context
                )
                .is_err()
        );
    }
    assert!(
        registry
            .register_interface_table(key("table_binding"), &witness, &object, &owned, &owned)
            .is_ok()
    );
}
