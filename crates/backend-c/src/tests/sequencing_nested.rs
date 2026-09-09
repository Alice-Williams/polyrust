//! All-syntax traversal rejects calls in unselected branches and unreachable code.
use super::contextual_reconstruction::{fixture, key, package};
use super::sequencing_roots::{check, int, predicate_call};
use super::*;
use crate::dialect::CKnownCall;

#[test]
fn nested_wrapped_sibling_and_argument_calls_are_rejected_even_after_return() {
    let (registry, file, function, scope) = fixture();
    let values = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let call = || predicate_call(&values);
    let boolean_call = || {
        values
            .numeric_conversion(CScalarType::Bool, call())
            .unwrap()
    };
    let double_call = || {
        values
            .call_value(
                values.known(CKnownCall::FloatTruncate),
                vec![
                    values
                        .numeric_conversion(CScalarType::F64, int(&values))
                        .unwrap(),
                ],
            )
            .unwrap()
    };
    let cases = [
        values
            .numeric_conversion(CScalarType::Bool, call())
            .unwrap(),
        values.binary(CBinaryOperator::Add, call(), call()).unwrap(),
        values
            .conditional(
                values.literal(CLiteral::Bool(false)).unwrap(),
                call(),
                int(&values),
            )
            .unwrap(),
        values
            .conditional(
                values.literal(CLiteral::Bool(true)).unwrap(),
                int(&values),
                call(),
            )
            .unwrap(),
        values
            .binary(
                CBinaryOperator::LogicalAnd,
                values.literal(CLiteral::Bool(false)).unwrap(),
                boolean_call(),
            )
            .unwrap(),
        values
            .binary(
                CBinaryOperator::LogicalOr,
                values.literal(CLiteral::Bool(true)).unwrap(),
                boolean_call(),
            )
            .unwrap(),
        values
            .call_value(values.known(CKnownCall::IsNan), vec![double_call()])
            .unwrap(),
        values
            .call_value(
                values.known(CKnownCall::FloatRemainder),
                vec![double_call(), double_call()],
            )
            .unwrap(),
    ];
    for value in cases {
        for unreachable in [false, true] {
            let mut body = Vec::new();
            if unreachable {
                body.push(statements.return_statement(None).unwrap());
            }
            body.push(statements.discard(value.clone()).unwrap());
            check(
                &registry,
                package(
                    &registry,
                    file.clone(),
                    function.clone(),
                    scope.clone(),
                    body,
                ),
                true,
            );
        }
    }
    check(
        &registry,
        package(
            &registry,
            file,
            function,
            scope,
            vec![
                statements
                    .discard(
                        values
                            .binary(CBinaryOperator::Add, int(&values), int(&values))
                            .unwrap(),
                    )
                    .unwrap(),
            ],
        ),
        false,
    );
}

#[test]
fn calls_in_if_loop_and_switch_conditions_are_not_root_positions() {
    for kind in 0..3 {
        for hidden in [false, true] {
            let (mut registry, file, function, scope) = fixture();
            let first = registry
                .register_scope(&function, Some(&scope), key("first"))
                .unwrap();
            let second = (kind != 1).then(|| {
                registry
                    .register_scope(&function, Some(&scope), key("second"))
                    .unwrap()
            });
            let iteration =
                (kind == 1).then(|| registry.register_loop(&scope, key("iteration")).unwrap());
            let switch =
                (kind == 2).then(|| registry.register_switch(&scope, key("choice")).unwrap());
            let size = CObjectType::scalar(CScalarType::Size);
            let counter = (kind == 1).then(|| {
                registry
                    .register_local(&scope, key("counter"), size.clone())
                    .unwrap()
            });
            let bound = (kind == 1).then(|| {
                registry
                    .register_local(
                        &scope,
                        key("bound"),
                        size.with_constness(CConstness::Const).unwrap(),
                    )
                    .unwrap()
            });
            let values = CExpressions::new(&registry);
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let condition = if hidden {
                predicate_call(&values)
            } else {
                int(&values)
            };
            let boolean = values
                .numeric_conversion(CScalarType::Bool, condition.clone())
                .unwrap();
            let mut body = Vec::new();
            let statement = match kind {
                0 => ast
                    .if_statement(
                        boolean,
                        ast.block(first, vec![]).unwrap(),
                        ast.block(second.unwrap(), vec![]).unwrap(),
                    )
                    .unwrap(),
                1 => {
                    let initial = values
                        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
                        .unwrap();
                    for local in [counter.as_ref().unwrap(), bound.as_ref().unwrap()] {
                        body.push(
                            ast.declare(
                                local.clone(),
                                Some(values.expression_initializer(initial.clone()).unwrap()),
                            )
                            .unwrap(),
                        );
                    }
                    ast.counted_loop(
                        iteration.unwrap(),
                        counter.unwrap(),
                        bound.unwrap(),
                        boolean,
                        ast.block(first, vec![]).unwrap(),
                    )
                    .unwrap()
                }
                2 => {
                    let switch = switch.unwrap();
                    let exit = ast
                        .break_statement(CBreakTarget::Switch(switch.clone()))
                        .unwrap();
                    ast.switch_statement(
                        switch,
                        condition,
                        vec![
                            ast.switch_arm(
                                vec![CCaseConstant::Signed(CSignedLiteral::Int(1))],
                                ast.block(first, vec![exit.clone()]).unwrap(),
                            )
                            .unwrap(),
                        ],
                        ast.block(second.unwrap(), vec![exit]).unwrap(),
                    )
                    .unwrap()
                }
                _ => unreachable!(),
            };
            body.push(statement);
            check(
                &registry,
                package(&registry, file, function, scope, body),
                hidden,
            );
        }
    }
}

#[test]
fn local_array_struct_and_union_initializer_children_must_be_call_free() {
    for aggregate in 0..3 {
        for hidden in [false, true] {
            let (mut registry, file, function, scope) = fixture();
            let scalar = CObjectType::scalar(CScalarType::Int);
            let owner = match aggregate {
                1 => Some(CAggregateRef::Struct(
                    registry.declare_struct(&file, key("Record")).unwrap(),
                )),
                2 => Some(CAggregateRef::Union(
                    registry.declare_union(&file, key("Choice")).unwrap(),
                )),
                _ => None,
            };
            let member = owner.as_ref().map(|owner| {
                registry
                    .register_member(owner, key("value"), scalar.clone())
                    .unwrap()
            });
            if let Some(owner) = &owner {
                registry
                    .define_aggregate(owner, vec![member.clone().unwrap()])
                    .unwrap();
            }
            let ty = match &owner {
                Some(CAggregateRef::Struct(value)) => CObjectType::structure(value.clone()),
                Some(CAggregateRef::Union(value)) => CObjectType::union(value.clone()),
                None => CObjectType::array(scalar, CArrayLength::new(1).unwrap()).unwrap(),
            };
            let local = registry
                .register_local(&scope, key("storage"), ty.clone())
                .unwrap();
            let values = CExpressions::new(&registry);
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let value = if hidden {
                predicate_call(&values)
            } else {
                int(&values)
            };
            let leaf = values.expression_initializer(value).unwrap();
            let initializer = match &owner {
                Some(CAggregateRef::Struct(value)) => values
                    .struct_initializer(value.clone(), vec![(member.unwrap(), leaf)])
                    .unwrap(),
                Some(CAggregateRef::Union(value)) => values
                    .union_initializer(value.clone(), member.unwrap(), leaf)
                    .unwrap(),
                None => values.array_initializer(ty, vec![leaf]).unwrap(),
            };
            let mut source = package(
                &registry,
                file.clone(),
                function,
                scope,
                vec![ast.declare(local, Some(initializer)).unwrap()],
            );
            if let Some(owner) = owner {
                source.items.insert(
                    0,
                    CFileItem::Declaration(
                        CDeclarations::new(&registry, file)
                            .unwrap()
                            .aggregate(owner)
                            .unwrap(),
                    ),
                );
            }
            check(&registry, source, hidden);
        }
    }
}
