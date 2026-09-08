//! Actual control occurrences/targets, not just valid registration categories.

use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

#[derive(Clone, Copy, Debug)]
enum Kind {
    Loop,
    Switch,
}
#[derive(Clone, Copy, Debug)]
enum Mutation {
    Duplicate,
    Delete,
    Cross,
    WrongContainingScope,
    ForeignFunction,
}

fn root(source: &mut CSourceFile) -> &mut CBlock {
    let CFileItem::Definition(value) = &mut source.items[0] else {
        unreachable!()
    };
    let CDefinitionKind::Function { body, .. } = &mut value.kind else {
        unreachable!()
    };
    body
}

#[test]
fn control_occurrences_cannot_be_duplicated_deleted_crossed_or_reowned() {
    for kind in [Kind::Loop, Kind::Switch] {
        for mutation in [
            Mutation::Duplicate,
            Mutation::Delete,
            Mutation::Cross,
            Mutation::WrongContainingScope,
            Mutation::ForeignFunction,
        ] {
            let (mut registry, file, function, scope) = fixture();
            let children = ["first", "second"].map(|name| {
                registry
                    .register_scope(&function, Some(&scope), key(name))
                    .unwrap()
            });
            let size = CObjectType::scalar(CScalarType::Size);
            let counter = registry
                .register_local(&scope, key("counter"), size.clone())
                .unwrap();
            let bound = registry
                .register_local(
                    &scope,
                    key("bound"),
                    size.with_constness(CConstness::Const).unwrap(),
                )
                .unwrap();
            let loop_ids = if matches!(kind, Kind::Loop) {
                ["first", "second"]
                    .map(|name| Some(registry.register_loop(&scope, key(name)).unwrap()))
            } else {
                [None, None]
            };
            let switch_ids = if matches!(kind, Kind::Switch) {
                ["first", "second"]
                    .map(|name| Some(registry.register_switch(&scope, key(name)).unwrap()))
            } else {
                [None, None]
            };
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let values = CExpressions::new(&registry);
            let zero = values
                .expression_initializer(
                    values
                        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
                        .unwrap(),
                )
                .unwrap();
            let mut statements = vec![
                ast.declare(counter.clone(), Some(zero.clone())).unwrap(),
                ast.declare(bound.clone(), Some(zero)).unwrap(),
            ];
            for (i, child) in children.into_iter().enumerate() {
                let statement = match kind {
                    Kind::Loop => {
                        let id = loop_ids[i].clone().unwrap();
                        let body = ast
                            .block(child, vec![ast.continue_statement(id.clone()).unwrap()])
                            .unwrap();
                        ast.counted_loop(
                            id,
                            counter.clone(),
                            bound.clone(),
                            values.literal(CLiteral::Bool(false)).unwrap(),
                            body,
                        )
                        .unwrap()
                    }
                    Kind::Switch => {
                        let id = switch_ids[i].clone().unwrap();
                        let body = ast
                            .block(
                                child,
                                vec![
                                    ast.break_statement(CBreakTarget::Switch(id.clone()))
                                        .unwrap(),
                                ],
                            )
                            .unwrap();
                        ast.switch_statement(id, int(&values), vec![], body)
                            .unwrap()
                    }
                };
                statements.push(statement);
            }
            let mut source = package(&registry, file, function, scope, statements);
            registry.check_context(&[source.clone()]).unwrap();
            let body = root(&mut source);
            match mutation {
                Mutation::Duplicate => body.statements.push(body.statements[2].clone()),
                Mutation::Delete => {
                    body.statements.pop();
                }
                Mutation::Cross => {
                    for i in 0..2 {
                        match &mut body.statements[i + 2].kind {
                            CStatementKind::BoundedLoop { identity, .. } => {
                                *identity = loop_ids[1 - i].clone().unwrap()
                            }
                            CStatementKind::Switch { identity, .. } => {
                                *identity = switch_ids[1 - i].clone().unwrap()
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                Mutation::WrongContainingScope => {
                    let second = body.statements.pop().unwrap();
                    let nested = match &mut body.statements[2].kind {
                        CStatementKind::BoundedLoop { body, .. } => body,
                        CStatementKind::Switch { default, .. } => default,
                        _ => unreachable!(),
                    };
                    nested.statements.insert(0, second);
                }
                Mutation::ForeignFunction => body.statements[2].function = fixture().2,
            }
            assert!(
                registry.check_context(&[source]).is_err(),
                "{kind:?}/{mutation:?}"
            );
        }
    }
}

#[test]
fn nested_loops_require_the_innermost_break_and_continue() {
    for use_continue in [false, true] {
        for target_outer in [false, true] {
            let (mut registry, file, function, scope) = fixture();
            let outer_body = registry
                .register_scope(&function, Some(&scope), key("outer_body"))
                .unwrap();
            let inner_body = registry
                .register_scope(&function, Some(&outer_body), key("inner_body"))
                .unwrap();
            let outer = registry.register_loop(&scope, key("outer")).unwrap();
            let inner = registry.register_loop(&outer_body, key("inner")).unwrap();
            let locals = [&scope, &outer_body].map(|scope| {
                let size = CObjectType::scalar(CScalarType::Size);
                (
                    registry
                        .register_local(scope, key("counter"), size.clone())
                        .unwrap(),
                    registry
                        .register_local(
                            scope,
                            key("bound"),
                            size.with_constness(CConstness::Const).unwrap(),
                        )
                        .unwrap(),
                )
            });
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let values = CExpressions::new(&registry);
            let target = if target_outer {
                outer.clone()
            } else {
                inner.clone()
            };
            let transfer = if use_continue {
                ast.continue_statement(target).unwrap()
            } else {
                ast.break_statement(CBreakTarget::Loop(target)).unwrap()
            };
            let inner_statement = ast
                .counted_loop(
                    inner,
                    locals[1].0.clone(),
                    locals[1].1.clone(),
                    values.literal(CLiteral::Bool(false)).unwrap(),
                    ast.block(inner_body, vec![transfer]).unwrap(),
                )
                .unwrap();
            let declarations = |pair: &(CLocalRef, CLocalRef)| {
                let zero = values
                    .expression_initializer(
                        values
                            .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
                            .unwrap(),
                    )
                    .unwrap();
                vec![
                    ast.declare(pair.0.clone(), Some(zero.clone())).unwrap(),
                    ast.declare(pair.1.clone(), Some(zero)).unwrap(),
                ]
            };
            let mut nested = declarations(&locals[1]);
            nested.push(inner_statement);
            let outer_statement = ast
                .counted_loop(
                    outer,
                    locals[0].0.clone(),
                    locals[0].1.clone(),
                    values.literal(CLiteral::Bool(false)).unwrap(),
                    ast.block(outer_body, nested).unwrap(),
                )
                .unwrap();
            let mut body = declarations(&locals[0]);
            body.push(outer_statement);
            let result = registry.check_context(&[package(&registry, file, function, scope, body)]);
            assert_eq!(
                result,
                if target_outer {
                    Err(CContextError::WrongControlTarget)
                } else {
                    Ok(())
                }
            );
        }
    }
}
