//! Registered adapter conversions and void calls retain their proof subjects.
use super::contextual_reconstruction::{fixture, int, key, package};
use super::contextual_value_variants::prove;
use super::*;
use portable_core_ir::CoreDeclaration;

#[test]
fn both_adapter_conversion_branches_rebuild_the_registered_subject() {
    let core = super::contextual_origins::core();
    let implementation = core
        .module()
        .declarations
        .iter()
        .find_map(|d| match d {
            CoreDeclaration::Implementation(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    let (mut registry, file, function, scope) = fixture();
    let record = registry.declare_struct(&file, key("Record")).unwrap();
    let interface = registry.declare_struct(&file, key("Interface")).unwrap();
    let table_type = registry.declare_struct(&file, key("Table")).unwrap();
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
            CObjectType::structure(table_type)
                .with_constness(CConstness::Const)
                .unwrap(),
        )
        .unwrap();
    let table = registry
        .register_interface_table(key("TableBinding"), &witness, &object, &function, &function)
        .unwrap();
    let adapter = registry
        .register_interface_adapter(key("Adapter"), &table)
        .unwrap();
    let values = CExpressions::new(&registry);
    for qualifier in [CConstness::Unqualified, CConstness::Const] {
        let ty = CObjectType::pointer(CPointerTarget::Object(Box::new(
            CObjectType::structure(record.clone())
                .with_constness(qualifier)
                .unwrap(),
        )));
        let pointer = values
            .literal(CLiteral::NullPointer(CNullPointer::new(ty).unwrap()))
            .unwrap();
        let erased = values.adapter_erase(adapter.clone(), pointer).unwrap();
        let restored = values
            .adapter_restore(adapter.clone(), erased.clone())
            .unwrap();
        prove(&registry, &file, &function, &scope, erased);
        prove(&registry, &file, &function, &scope, restored);
    }
    // This is reconstruction evidence only; this deliberately minimal fixture
    // is not an ownership/callback ABI certificate or a generated executable.
}

#[test]
fn labeled_effect_calls_recheck_arguments_contracts_and_callable_brands() {
    let (mut registry, file, function, scope) = fixture();
    let callee = registry
        .register_function(
            &file,
            key("callee"),
            CFunctionType::new(
                CReturnType::Void,
                vec![CParameterType::new(CObjectType::scalar(CScalarType::I32)).unwrap()],
            ),
        )
        .unwrap();
    let label = registry
        .register_cleanup_exit(&scope, key("cleanup"))
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let (foreign, _, foreign_function, _) = fixture();
    let foreign_callable = CExpressions::new(&foreign)
        .direct(foreign_function)
        .unwrap();
    for indirect in [false, true] {
        let callable = if indirect {
            values
                .indirect(
                    values.function_address(callee.clone()).unwrap(),
                    callee.clone(),
                )
                .unwrap()
        } else {
            values.direct(callee.clone()).unwrap()
        };
        let effect = values.call_effect(callable, vec![int(&values)]).unwrap();
        let make = |effect| {
            package(
                &registry,
                file.clone(),
                function.clone(),
                scope.clone(),
                vec![
                    ast.label(label.clone(), ast.evaluate(effect).unwrap())
                        .unwrap(),
                ],
            )
        };
        registry
            .check_local_structure(&[make(effect.clone())])
            .unwrap();
        for mutation in 0..3 {
            let mut bad = effect.clone();
            match mutation {
                0 => {
                    bad.call.arguments.clear();
                }
                1 => bad.call.arguments[0].ty = CObjectType::scalar(CScalarType::F64),
                2 => bad.call.callable.brand = foreign_callable.brand.clone(),
                _ => unreachable!(),
            }
            assert!(registry.check_local_structure(&[make(bad)]).is_err());
        }
    }
}
