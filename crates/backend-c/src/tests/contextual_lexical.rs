//! Lexical checks reject invalid unreachable syntax independently of flow.

use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

fn scalar() -> CObjectType {
    CObjectType::scalar(CScalarType::I32)
}

#[test]
fn local_visibility_follows_declaration_order_even_after_return() {
    let (mut registry, file, function, scope) = fixture();
    let local = registry
        .register_local(&scope, key("value"), scalar())
        .unwrap();
    let values = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let declaration = statements.declare(local.clone(), None).unwrap();
    let read = statements
        .discard(values.read(values.local(local).unwrap()).unwrap())
        .unwrap();
    let source = |body| {
        package(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            body,
        )
    };
    registry
        .check_lexical_structure(&[source(vec![declaration.clone(), read.clone()])])
        .unwrap();
    assert_eq!(
        registry.check_lexical_structure(&[source(vec![read.clone(), declaration.clone()])]),
        Err(CContextError::InvisibleBinding)
    );
    assert_eq!(
        registry.check_lexical_structure(&[source(vec![
            statements.return_statement(None).unwrap(),
            read,
            declaration
        ])]),
        Err(CContextError::InvisibleBinding)
    );
}

#[test]
fn descendant_binding_does_not_leak_and_ancestor_binding_remains_visible() {
    let (mut registry, file, function, scope) = fixture();
    let child = registry
        .register_scope(&function, Some(&scope), key("child"))
        .unwrap();
    let outer = registry
        .register_local(&scope, key("value"), scalar())
        .unwrap();
    let inner = registry
        .register_local(&child, key("value"), scalar())
        .unwrap();
    let values = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let read_outer = statements
        .discard(values.read(values.local(outer.clone()).unwrap()).unwrap())
        .unwrap();
    let read_inner = statements
        .discard(values.read(values.local(inner.clone()).unwrap()).unwrap())
        .unwrap();
    let nested = statements
        .nested_block(
            statements
                .block(
                    child,
                    vec![
                        statements.declare(inner, None).unwrap(),
                        read_outer,
                        read_inner.clone(),
                    ],
                )
                .unwrap(),
        )
        .unwrap();
    let declaration = statements.declare(outer, None).unwrap();
    let source = |body| {
        package(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            body,
        )
    };
    registry
        .check_lexical_structure(&[source(vec![declaration.clone(), nested.clone()])])
        .unwrap();
    assert_eq!(
        registry.check_lexical_structure(&[source(vec![declaration, nested, read_inner])]),
        Err(CContextError::InvisibleBinding)
    );
}

#[test]
fn cleanup_exits_are_forward_and_cannot_skip_declarations() {
    let (mut registry, file, function, scope) = fixture();
    let label = registry
        .register_cleanup_exit(&scope, key("cleanup"))
        .unwrap();
    let local = registry
        .register_local(&scope, key("value"), scalar())
        .unwrap();
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let jump = statements.cleanup_jump(label.clone()).unwrap();
    let label = statements.label(label, statements.empty()).unwrap();
    let declaration = statements.declare(local, None).unwrap();
    let source = |body| {
        package(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            body,
        )
    };
    registry
        .check_lexical_structure(&[source(vec![
            declaration.clone(),
            jump.clone(),
            label.clone(),
        ])])
        .unwrap();
    assert_eq!(
        registry.check_lexical_structure(&[source(vec![
            jump.clone(),
            declaration.clone(),
            label.clone()
        ])]),
        Err(CContextError::InvalidCleanupExit)
    );
    assert_eq!(
        registry.check_lexical_structure(&[source(vec![declaration, label, jump])]),
        Err(CContextError::InvalidCleanupExit)
    );
}

#[test]
fn cleanup_can_leave_a_child_scope_but_cannot_enter_one() {
    for jump_into_child in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let child = registry
            .register_scope(&function, Some(&scope), key("child"))
            .unwrap();
        let label_scope = if jump_into_child { &child } else { &scope };
        let label = registry
            .register_cleanup_exit(label_scope, key("cleanup"))
            .unwrap();
        let statements = CStatements::new(&registry, function.clone()).unwrap();
        let jump = statements.cleanup_jump(label.clone()).unwrap();
        let label = statements.label(label, statements.empty()).unwrap();
        let body = if jump_into_child {
            vec![
                jump,
                statements
                    .nested_block(statements.block(child, vec![label]).unwrap())
                    .unwrap(),
            ]
        } else {
            vec![
                statements
                    .nested_block(statements.block(child, vec![jump]).unwrap())
                    .unwrap(),
                label,
            ]
        };
        let result =
            registry.check_lexical_structure(&[package(&registry, file, function, scope, body)]);
        if jump_into_child {
            assert_eq!(result, Err(CContextError::InvalidCleanupExit));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn continue_crosses_a_switch_but_break_cannot_skip_it() {
    for use_continue in [true, false] {
        let (mut registry, file, function, scope) = fixture();
        let body_scope = registry
            .register_scope(&function, Some(&scope), key("body"))
            .unwrap();
        let arm_scope = registry
            .register_scope(&function, Some(&body_scope), key("arm"))
            .unwrap();
        let default_scope = registry
            .register_scope(&function, Some(&body_scope), key("default_arm"))
            .unwrap();
        let loop_id = registry.register_loop(&scope, key("loop")).unwrap();
        let switch = registry
            .register_switch(&body_scope, key("selection"))
            .unwrap();
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
        let values = CExpressions::new(&registry);
        let statements = CStatements::new(&registry, function.clone()).unwrap();
        let transfer = if use_continue {
            statements.continue_statement(loop_id.clone()).unwrap()
        } else {
            statements
                .break_statement(CBreakTarget::Loop(loop_id.clone()))
                .unwrap()
        };
        let arm = statements
            .switch_arm(
                vec![CCaseConstant::Signed(CSignedLiteral::Int(0))],
                statements.block(arm_scope, vec![transfer]).unwrap(),
            )
            .unwrap();
        let switch = statements
            .switch_statement(
                switch,
                int(&values),
                vec![arm],
                statements.block(default_scope, vec![]).unwrap(),
            )
            .unwrap();
        let body = statements.block(body_scope, vec![switch]).unwrap();
        let loop_statement = statements
            .counted_loop(
                loop_id,
                counter.clone(),
                bound.clone(),
                values.literal(CLiteral::Bool(false)).unwrap(),
                body,
            )
            .unwrap();
        let size_zero = values
            .expression_initializer(
                values
                    .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
                    .unwrap(),
            )
            .unwrap();
        let declarations = vec![
            statements
                .declare(counter, Some(size_zero.clone()))
                .unwrap(),
            statements.declare(bound, Some(size_zero)).unwrap(),
        ];
        let mut body = declarations.clone();
        body.push(loop_statement.clone());
        let result = registry.check_lexical_structure(&[package(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            body,
        )]);
        if use_continue {
            result.unwrap();
            let mut before_declarations = vec![loop_statement];
            before_declarations.extend(declarations);
            assert_eq!(
                registry.check_lexical_structure(&[package(
                    &registry,
                    file,
                    function,
                    scope,
                    before_declarations
                )]),
                Err(CContextError::InvisibleBinding)
            );
        } else {
            assert_eq!(result, Err(CContextError::WrongControlTarget));
        }
    }
}
