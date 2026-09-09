//! Callable expressions and all argument positions remain call-free.
use super::contextual_reconstruction::{fixture, key, package};
use super::sequencing_roots::{check, int, predicate_call};
use super::*;

#[test]
fn a_parameter_place_is_not_a_direct_local_call_result_destination() {
    for hidden in [false, true] {
        let mut registry = CRegistry::new();
        let file = registry
            .register_file(CFileKey {
                path: portable_codegen::RelativeOutputPath::new("src/parameter.c").unwrap(),
                role: CFileRole::TestSource,
            })
            .unwrap();
        let function = registry
            .register_function(
                &file,
                key("run"),
                CFunctionType::new(
                    CReturnType::Void,
                    vec![CParameterType::new(CObjectType::scalar(CScalarType::Int)).unwrap()],
                ),
            )
            .unwrap();
        let parameter = registry
            .register_parameter(&function, 0, key("input"), CConstness::Unqualified)
            .unwrap();
        let scope = registry
            .register_scope(&function, None, key("root"))
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let value = if hidden {
            predicate_call(&values)
        } else {
            int(&values)
        };
        let body = ast
            .block(
                scope,
                vec![
                    ast.assign(values.parameter(parameter.clone()).unwrap(), value)
                        .unwrap(),
                ],
            )
            .unwrap();
        let declarations = CDeclarations::new(&registry, file).unwrap();
        let source = declarations
            .source_file(vec![CFileItem::Definition(
                declarations
                    .function_definition(function, CLinkage::External, vec![parameter], body)
                    .unwrap(),
            )])
            .unwrap();
        check(&registry, source, hidden);
    }
}

#[test]
fn direct_and_indirect_calls_reject_nested_arguments_for_values_and_void_effects() {
    for indirect in [false, true] {
        for effect in [false, true] {
            for hidden in [false, true] {
                let (mut registry, file, function, scope) = fixture();
                let scalar = CObjectType::scalar(CScalarType::Int);
                let result = if effect {
                    CReturnType::Void
                } else {
                    CReturnType::Value(CReturnValue::new(scalar.clone()).unwrap())
                };
                let callee = registry
                    .register_function(
                        &file,
                        key("callee"),
                        CFunctionType::new(result, vec![CParameterType::new(scalar).unwrap()]),
                    )
                    .unwrap();
                let parameter = registry
                    .register_parameter(&callee, 0, key("input"), CConstness::Unqualified)
                    .unwrap();
                let child_scope = registry
                    .register_scope(&callee, None, key("callee_root"))
                    .unwrap();
                let values = CExpressions::new(&registry);
                let ast = CStatements::new(&registry, function.clone()).unwrap();
                let target_ast = CStatements::new(&registry, callee.clone()).unwrap();
                let declarations = CDeclarations::new(&registry, file.clone()).unwrap();
                let target_body = target_ast
                    .block(
                        child_scope,
                        vec![
                            target_ast
                                .return_statement(if effect {
                                    None
                                } else {
                                    Some(
                                        values
                                            .read(values.parameter(parameter.clone()).unwrap())
                                            .unwrap(),
                                    )
                                })
                                .unwrap(),
                        ],
                    )
                    .unwrap();
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
                let argument = if hidden {
                    predicate_call(&values)
                } else {
                    int(&values)
                };
                let statement = if effect {
                    ast.evaluate(values.call_effect(callable, vec![argument]).unwrap())
                        .unwrap()
                } else {
                    ast.discard(values.call_value(callable, vec![argument]).unwrap())
                        .unwrap()
                };
                let mut source = package(&registry, file, function, scope, vec![statement]);
                source.items.push(CFileItem::Definition(
                    declarations
                        .function_definition(
                            callee,
                            CLinkage::External,
                            vec![parameter],
                            target_body,
                        )
                        .unwrap(),
                ));
                check(&registry, source, hidden);
            }
        }
    }
}

#[test]
fn indirect_callable_operand_cannot_hide_a_call_returning_the_right_function_pointer_type() {
    for hidden in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let pointer_type = CObjectType::pointer(CPointerTarget::Function(Box::new(
            function.signature().clone(),
        )));
        let factory = registry
            .register_function(
                &file,
                key("factory"),
                CFunctionType::new(
                    CReturnType::Value(CReturnValue::new(pointer_type).unwrap()),
                    vec![],
                ),
            )
            .unwrap();
        let factory_scope = registry
            .register_scope(&factory, None, key("factory_root"))
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let target_ast = CStatements::new(&registry, factory.clone()).unwrap();
        let address = values.function_address(function.clone()).unwrap();
        let factory_body = target_ast
            .block(
                factory_scope,
                vec![target_ast.return_statement(Some(address.clone())).unwrap()],
            )
            .unwrap();
        let pointer = if hidden {
            values
                .call_value(values.direct(factory.clone()).unwrap(), vec![])
                .unwrap()
        } else {
            address
        };
        let callable = values.indirect(pointer, function.clone()).unwrap();
        let statement = ast
            .evaluate(values.call_effect(callable, vec![]).unwrap())
            .unwrap();
        let mut source = package(&registry, file.clone(), function, scope, vec![statement]);
        source.items.push(CFileItem::Definition(
            CDeclarations::new(&registry, file)
                .unwrap()
                .function_definition(factory, CLinkage::External, vec![], factory_body)
                .unwrap(),
        ));
        check(&registry, source, hidden);
    }
}

#[test]
fn calls_inside_pointer_dereferences_and_pointer_index_bases_are_not_roots() {
    for index in [false, true] {
        for hidden in [false, true] {
            let (mut registry, file, function, scope) = fixture();
            let scalar = CObjectType::scalar(CScalarType::Int);
            let object = registry
                .register_object(&file, key("storage"), scalar.clone())
                .unwrap();
            let pointer_type =
                CObjectType::pointer(CPointerTarget::Object(Box::new(scalar.clone())));
            let factory = registry
                .register_function(
                    &file,
                    key("factory"),
                    CFunctionType::new(
                        CReturnType::Value(CReturnValue::new(pointer_type).unwrap()),
                        vec![],
                    ),
                )
                .unwrap();
            let factory_scope = registry
                .register_scope(&factory, None, key("factory_root"))
                .unwrap();
            let values = CExpressions::new(&registry);
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let target_ast = CStatements::new(&registry, factory.clone()).unwrap();
            let address = values
                .address_of(values.global(object.clone()).unwrap())
                .unwrap();
            let factory_body = target_ast
                .block(
                    factory_scope,
                    vec![target_ast.return_statement(Some(address.clone())).unwrap()],
                )
                .unwrap();
            let pointer = if hidden {
                values
                    .call_value(values.direct(factory.clone()).unwrap(), vec![])
                    .unwrap()
            } else {
                address
            };
            let place = if index {
                values
                    .index(
                        CIndexBase::Pointer(Box::new(pointer)),
                        values
                            .literal(CLiteral::Signed(CSignedLiteral::Int(0)))
                            .unwrap(),
                    )
                    .unwrap()
            } else {
                values.dereference(pointer).unwrap()
            };
            let mut source = package(
                &registry,
                file.clone(),
                function,
                scope,
                vec![ast.discard(values.read(place).unwrap()).unwrap()],
            );
            let declarations = CDeclarations::new(&registry, file).unwrap();
            source.items.insert(
                0,
                CFileItem::Definition(
                    declarations
                        .object_definition(
                            object,
                            CLinkage::External,
                            values.zero_initializer(scalar).unwrap(),
                        )
                        .unwrap(),
                ),
            );
            source.items.push(CFileItem::Definition(
                declarations
                    .function_definition(factory, CLinkage::External, vec![], factory_body)
                    .unwrap(),
            ));
            check(&registry, source, hidden);
        }
    }
}
