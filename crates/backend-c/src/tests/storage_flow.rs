//! Common-path facts, expression selection, loop convergence and activation reuse.
use super::{
    contextual_reconstruction::key, counted_fixture, index_extent_fixture as arrays,
    numeric_fixture::Fixture, storage_fixture::*, *,
};

#[test]
fn branch_join_keeps_initialization_only_when_every_path_writes() {
    for write_else in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Bool]);
        let value = f.local(CScalarType::Int, "value");
        let pointer = local(&mut f, pointer_type(value.ty().clone()), "pointer");
        let write = f
            .ast()
            .assign(f.values().dereference(f.read(&pointer)).unwrap(), f.int(7))
            .unwrap();
        let branch = f.branch(
            f.input(0),
            vec![write.clone()],
            if write_else { vec![write] } else { vec![] },
        );
        check(
            &f,
            vec![
                f.ast().declare(value.clone(), None).unwrap(),
                f.declare(&pointer, address(&f, f.values().local(value).unwrap())),
                branch,
                f.discard(pointed_read(&f, f.read(&pointer))),
            ],
            if write_else {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}

#[test]
fn conditional_and_short_circuit_children_use_their_actual_numeric_guard() {
    for conditional in [false, true] {
        for bound in [2, 3] {
            let mut f = Fixture::new(&[CScalarType::Size]);
            let array = arrays::array(&mut f, 2, "array");
            let guard = f.compare(CBinaryOperator::Less, f.input(0), f.size(bound));
            let read = pointed_read(&f, address(&f, arrays::place(&f, &array, f.input(0))));
            let expression = if conditional {
                f.values().conditional(guard, read, f.int(0)).unwrap()
            } else {
                f.binary(CBinaryOperator::LogicalAnd, guard, f.boolean(read))
            };
            check(
                &f,
                vec![arrays::declare(&f, &array), f.discard(expression)],
                if bound == 2 {
                    Ok(())
                } else {
                    Err(CSafetyError::IndexOutOfBounds)
                },
            );
        }
    }
}

#[test]
fn proved_null_tests_select_only_the_actual_safe_branch() {
    for nonnull in [false, true] {
        let mut f = Fixture::new(&[]);
        let value = f.local(CScalarType::Int, "value");
        let pointer = local(&mut f, pointer_type(value.ty().clone()), "pointer");
        let initial = if nonnull {
            address(&f, f.values().local(value.clone()).unwrap())
        } else {
            f.values()
                .literal(CLiteral::NullPointer(
                    CNullPointer::new(pointer.ty().clone()).unwrap(),
                ))
                .unwrap()
        };
        let guard = f.boolean(
            f.values()
                .pointer_test(CPointerTest::IsNonNull(Box::new(f.read(&pointer))))
                .unwrap(),
        );
        let selected = f
            .values()
            .conditional(guard, pointed_read(&f, f.read(&pointer)), f.int(0))
            .unwrap();
        check(
            &f,
            vec![
                f.declare(&value, f.int(1)),
                f.declare(&pointer, initial),
                f.discard(selected),
            ],
            Ok(()),
        );
    }
}

#[test]
fn zero_initializer_and_explicit_null_keep_the_same_fact_at_a_join() {
    let mut f = Fixture::new(&[CScalarType::Bool]);
    let pointer = local(
        &mut f,
        pointer_type(CObjectType::scalar(CScalarType::Int)),
        "pointer",
    );
    let null = f
        .values()
        .literal(CLiteral::NullPointer(
            CNullPointer::new(pointer.ty().clone()).unwrap(),
        ))
        .unwrap();
    let assign = f
        .ast()
        .assign(f.values().local(pointer.clone()).unwrap(), null)
        .unwrap();
    let branch = f.branch(f.input(0), vec![assign], vec![]);
    let test = f
        .values()
        .pointer_test(CPointerTest::IsNull(Box::new(f.read(&pointer))))
        .unwrap();
    check(
        &f,
        vec![arrays::declare(&f, &pointer), branch, f.discard(test)],
        Ok(()),
    );
}

#[test]
fn repeated_lexical_declaration_does_not_revive_a_previous_activation_pointer() {
    for use_previous in [false, true] {
        let mut f = counted_fixture::Fixture::new();
        let ty = CObjectType::scalar(CScalarType::Int);
        let value = f
            .registry
            .register_local(&f.body, key("value"), ty.clone())
            .unwrap();
        let pointer = f
            .registry
            .register_local(&f.scope, key("pointer"), pointer_type(ty))
            .unwrap();
        let branch_scope = f.child(&f.body.clone(), "selected");
        let else_scope = f.child(&f.body.clone(), "otherwise");
        let e = f.expressions();
        let ast = f.statements();
        let mut body = vec![
            ast.declare(
                value.clone(),
                Some(e.zero_initializer(value.ty().clone()).unwrap()),
            )
            .unwrap(),
        ];
        let read = ast
            .discard(e.read(e.dereference(f.read(&pointer)).unwrap()).unwrap())
            .unwrap();
        let guard = e
            .numeric_conversion(
                CScalarType::Bool,
                e.binary(CBinaryOperator::Greater, f.read(&f.counter), f.size(0))
                    .unwrap(),
            )
            .unwrap();
        body.push(
            ast.if_statement(
                guard,
                ast.block(
                    branch_scope,
                    if use_previous {
                        vec![read.clone()]
                    } else {
                        vec![]
                    },
                )
                .unwrap(),
                ast.block(else_scope, vec![]).unwrap(),
            )
            .unwrap(),
        );
        body.push(
            ast.assign(
                e.local(pointer.clone()).unwrap(),
                e.address_of(e.local(value).unwrap()).unwrap(),
            )
            .unwrap(),
        );
        body.extend([read, f.step()]);
        let null = e
            .literal(CLiteral::NullPointer(
                CNullPointer::new(pointer.ty().clone()).unwrap(),
            ))
            .unwrap();
        let source = f.source(vec![
            f.declare(&pointer, null),
            f.declare(&f.counter, f.size(0)),
            f.declare(&f.bound, f.size(2)),
            f.iteration(body),
        ]);
        f.registry
            .check_numeric_flow(std::slice::from_ref(&source))
            .unwrap();
        let result = f.registry.check_storage_paths(&[source]);
        if use_previous {
            assert!(
                matches!(
                    result,
                    Err(CSafetyError::ExpiredStorage
                        | CSafetyError::UnprovedStorage
                        | CSafetyError::UninitializedStorage)
                ),
                "{result:?}"
            );
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}
