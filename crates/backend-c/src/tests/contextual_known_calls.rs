//! Known identities do not bypass reconstruction, visibility or initialization.
use super::contextual_reconstruction::{fixture, key, package};
use super::known_call_fixtures::{Row, rows};
use super::*;
use crate::dialect::CKnownCall;

fn statement(
    row: &Row,
    values: &CExpressions<'_>,
    ast: &CStatements<'_>,
    arguments: Vec<CValue>,
) -> CStatement {
    let callable = values.known(row.call);
    if row.result.is_none() {
        ast.evaluate(values.call_effect(callable, arguments).unwrap())
            .unwrap()
    } else {
        ast.discard(values.call_value(callable, arguments).unwrap())
            .unwrap()
    }
}

fn call_mut(source: &mut CSourceFile) -> &mut CCall {
    let CFileItem::Definition(definition) = &mut source.items[0] else {
        panic!("definition")
    };
    let CDefinitionKind::Function { body, .. } = &mut definition.kind else {
        panic!("function")
    };
    match &mut body.statements[0].kind {
        CStatementKind::Evaluate(effect) => &mut effect.call,
        CStatementKind::Discard(value) => {
            let CValueKind::Call(call) = &mut value.kind else {
                panic!("call")
            };
            call
        }
        _ => panic!("call statement"),
    }
}

#[test]
fn every_known_call_is_independently_reconstructed_without_a_generated_body() {
    for row in rows() {
        let (registry, file, function, scope) = fixture();
        let (foreign, _, _, _) = fixture();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let valid = package(
            &registry,
            file,
            function,
            scope,
            vec![statement(&row, &values, &ast, row.arguments(&values))],
        );
        // Diagnostic context only: null extents/streams still need 02D safety.
        registry
            .check_context(std::slice::from_ref(&valid))
            .unwrap();
        for mutation in 0..6 {
            let mut bad = valid.clone();
            let call = call_mut(&mut bad);
            match mutation {
                0 => call.arguments.clear(),
                1 => call.arguments[0].ty = CObjectType::scalar(CScalarType::Bool),
                2 => call.arguments[0].kind = CValueKind::Literal(CLiteral::Bool(false)),
                3 => call.callable.brand = CExpressions::new(&foreign).known(row.call).brand,
                4 => {
                    call.arguments[0].brand =
                        row.arguments(&CExpressions::new(&foreign))[0].brand.clone()
                }
                5 => {
                    call.callable.kind = CCallableKind::Known(if row.call == CKnownCall::Allocate {
                        CKnownCall::Release
                    } else {
                        CKnownCall::Allocate
                    })
                }
                _ => unreachable!(),
            }
            assert!(
                registry.check_context(&[bad]).is_err(),
                "{:?} mutation {mutation}",
                row.call
            );
        }
    }
}

#[test]
fn every_known_argument_requires_lexical_visibility_and_initialized_value_storage() {
    for row in rows() {
        for index in 0..row.parameters.len() {
            for (dominates, initialized) in [(true, true), (false, true), (true, false)] {
                let (mut registry, file, function, scope) = fixture();
                let local = registry
                    .register_local(&scope, key("argument"), row.parameters[index].clone())
                    .unwrap();
                let values = CExpressions::new(&registry);
                let ast = CStatements::new(&registry, function.clone()).unwrap();
                let mut arguments = row.arguments(&values);
                let initial = arguments[index].clone();
                arguments[index] = values.read(values.local(local.clone()).unwrap()).unwrap();
                let declaration = ast
                    .declare(
                        local,
                        initialized.then(|| values.expression_initializer(initial).unwrap()),
                    )
                    .unwrap();
                let call = statement(&row, &values, &ast, arguments);
                let body = if dominates {
                    vec![declaration, call]
                } else {
                    vec![call, declaration]
                };
                assert_eq!(
                    registry.check_context(&[package(&registry, file, function, scope, body)]),
                    if !dominates {
                        Err(CContextError::InvisibleBinding)
                    } else if !initialized {
                        Err(CContextError::UninitializedRead)
                    } else {
                        Ok(())
                    },
                    "{:?} operand {index}",
                    row.call
                );
            }
        }
    }
}

#[test]
fn copy_contract_metadata_alone_does_not_initialize_an_addressed_object() {
    for initialized in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let ty = CObjectType::scalar(CScalarType::I32);
        let local = registry
            .register_local(&scope, key("destination"), ty.clone())
            .unwrap();
        let source = registry.register_local(&scope, key("source"), ty).unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let place = values.local(local.clone()).unwrap();
        let read_place = values.local(source.clone()).unwrap();
        let writable = CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified));
        let readable = CObjectType::pointer(CPointerTarget::Void(CConstness::Const));
        let destination = values
            .object_to_void(writable.clone(), values.address_of(place.clone()).unwrap())
            .unwrap();
        let from = values
            .object_to_void(writable, values.address_of(read_place).unwrap())
            .unwrap();
        let from = values.add_const(readable, from).unwrap();
        let int = || {
            values
                .literal(CLiteral::Signed(CSignedLiteral::I32(3)))
                .unwrap()
        };
        let copy = values
            .call_value(
                values.known(CKnownCall::CopyBytes),
                vec![
                    destination,
                    from,
                    values
                        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(4)))
                        .unwrap(),
                ],
            )
            .unwrap();
        let body = vec![
            ast.declare(
                local,
                initialized.then(|| values.expression_initializer(int()).unwrap()),
            )
            .unwrap(),
            ast.declare(source, Some(values.expression_initializer(int()).unwrap()))
                .unwrap(),
            ast.discard(copy).unwrap(),
            ast.discard(values.read(place).unwrap()).unwrap(),
        ];
        assert_eq!(
            registry.check_context(&[package(&registry, file, function, scope, body)]),
            if initialized {
                Ok(())
            } else {
                Err(CContextError::UninitializedRead)
            }
        );
    }
}
