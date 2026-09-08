//! Independent identity/occurrence corruption after legal tree construction.

use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

fn body_mut(source: &mut CSourceFile) -> &mut CBlock {
    let CFileItem::Definition(definition) = &mut source.items[0] else {
        unreachable!()
    };
    let CDefinitionKind::Function { body, .. } = &mut definition.kind else {
        unreachable!()
    };
    body
}

#[derive(Clone, Copy, Debug)]
enum Mutation {
    SwapSiblingScopes,
    DuplicateScope,
    MissingScope,
    WrongParent,
    WrongLocalScope,
    WrongRoot,
    ForeignFunction,
}

#[test]
fn actual_scope_occurrences_and_declaration_owners_are_rechecked() {
    for mutation in [
        Mutation::SwapSiblingScopes,
        Mutation::DuplicateScope,
        Mutation::MissingScope,
        Mutation::WrongParent,
        Mutation::WrongLocalScope,
        Mutation::WrongRoot,
        Mutation::ForeignFunction,
    ] {
        let (mut registry, file, function, scope) = fixture();
        let left = registry
            .register_scope(&function, Some(&scope), key("left"))
            .unwrap();
        let right = registry
            .register_scope(&function, Some(&scope), key("right"))
            .unwrap();
        let a = registry
            .register_local(&left, key("slot"), CObjectType::scalar(CScalarType::I32))
            .unwrap();
        let b = registry
            .register_local(&right, key("slot"), CObjectType::scalar(CScalarType::I32))
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let make_block = |scope, local| {
            ast.nested_block(
                ast.block(
                    scope,
                    vec![
                        ast.declare(
                            local,
                            Some(values.expression_initializer(int(&values)).unwrap()),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            )
            .unwrap()
        };
        let mut source = package(
            &registry,
            file,
            function,
            scope,
            vec![make_block(left.clone(), a), make_block(right.clone(), b)],
        );
        registry.check_context(&[source.clone()]).unwrap();
        let body = body_mut(&mut source);
        match mutation {
            Mutation::SwapSiblingScopes => {
                let CStatementKind::Block(first) = &mut body.statements[0].kind else {
                    unreachable!()
                };
                first.scope = right;
                let CStatementKind::Block(second) = &mut body.statements[1].kind else {
                    unreachable!()
                };
                second.scope = left;
            }
            Mutation::DuplicateScope => body.statements.push(body.statements[0].clone()),
            Mutation::MissingScope => {
                body.statements.pop();
            }
            Mutation::WrongParent => {
                let second = body.statements.pop().unwrap();
                let CStatementKind::Block(first) = &mut body.statements[0].kind else {
                    unreachable!()
                };
                first.statements.push(second);
            }
            Mutation::WrongLocalScope => {
                let CStatementKind::Block(first) = &mut body.statements[0].kind else {
                    unreachable!()
                };
                let local = first.statements.pop().unwrap();
                let CStatementKind::Block(second) = &mut body.statements[1].kind else {
                    unreachable!()
                };
                second.statements.push(local);
            }
            Mutation::WrongRoot => body.scope = left,
            Mutation::ForeignFunction => body.statements[0].function = fixture().2,
        }
        assert!(registry.check_context(&[source]).is_err(), "{mutation:?}");
    }
}

#[test]
fn parameter_occurrences_retain_exact_owner_index_order_and_inventory() {
    #[derive(Clone, Copy, Debug)]
    enum Mutation {
        Drop,
        Duplicate,
        Swap,
        ForeignOwner,
    }
    for mutation in [
        Mutation::Drop,
        Mutation::Duplicate,
        Mutation::Swap,
        Mutation::ForeignOwner,
    ] {
        let mut registry = CRegistry::new();
        let file = registry
            .register_file(CFileKey {
                path: portable_codegen::RelativeOutputPath::new("src/parameters.c").unwrap(),
                role: CFileRole::TestSource,
            })
            .unwrap();
        let signature = CFunctionType::new(
            CReturnType::Void,
            vec![
                CParameterType::new(CObjectType::scalar(CScalarType::I32)).unwrap(),
                CParameterType::new(CObjectType::scalar(CScalarType::I32)).unwrap(),
            ],
        );
        let function = registry
            .register_function(&file, key("run"), signature.clone())
            .unwrap();
        let other = registry
            .register_function(&file, key("other"), signature)
            .unwrap();
        let scopes =
            [&function, &other].map(|f| registry.register_scope(f, None, key("root")).unwrap());
        let params = [&function, &other].map(|f| {
            ["first", "second"]
                .into_iter()
                .enumerate()
                .map(|(i, name)| {
                    registry
                        .register_parameter(f, i, key(name), CConstness::Unqualified)
                        .unwrap()
                })
                .collect::<Vec<_>>()
        });
        let declarations = CDeclarations::new(&registry, file).unwrap();
        let mut items = Vec::new();
        for ((f, scope), parameters) in [&function, &other].into_iter().zip(scopes).zip(&params) {
            let ast = CStatements::new(&registry, f.clone()).unwrap();
            let values = CExpressions::new(&registry);
            let body = ast
                .block(
                    scope,
                    vec![
                        ast.discard(
                            values
                                .read(values.parameter(parameters[0].clone()).unwrap())
                                .unwrap(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap();
            items.push(CFileItem::Definition(
                declarations
                    .function_definition(f.clone(), CLinkage::External, parameters.clone(), body)
                    .unwrap(),
            ));
        }
        let mut source = declarations.source_file(items).unwrap();
        registry.check_context(&[source.clone()]).unwrap();
        let CFileItem::Definition(definition) = &mut source.items[0] else {
            unreachable!()
        };
        let CDefinitionKind::Function { parameters, .. } = &mut definition.kind else {
            unreachable!()
        };
        match mutation {
            Mutation::Drop => {
                parameters.pop();
            }
            Mutation::Duplicate => parameters[1] = parameters[0].clone(),
            Mutation::Swap => parameters.swap(0, 1),
            Mutation::ForeignOwner => parameters[0] = params[1][0].clone(),
        }
        assert!(registry.check_context(&[source]).is_err(), "{mutation:?}");
    }
}
