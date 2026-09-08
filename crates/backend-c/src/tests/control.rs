//! Counted-loop payloads and structural control identities, not CFG proofs.

use super::{registry_nominals::key, statements::setup};
use crate::ast::{
    CBinaryOperator, CBreakTarget, CCaseConstant, CConstness, CCountedStep, CExpressions, CLiteral,
    CObjectType, CReturnType, CScalarType as T, CSignedLiteral, CStatementError as E,
    CStatementKind, CStatements, CUnsignedLiteral,
};

#[test]
fn counted_loop_retains_exact_counter_bound_condition_and_explicit_update() {
    let (mut registry, function, root) = setup(CReturnType::Void);
    let body_scope = registry
        .register_scope(&function, Some(&root), key("loop_body"))
        .unwrap();
    let counter = registry
        .register_local(&root, key("counter"), CObjectType::scalar(T::Size))
        .unwrap();
    let bound = registry
        .register_local(
            &root,
            key("bound"),
            CObjectType::scalar(T::Size)
                .with_constness(CConstness::Const)
                .unwrap(),
        )
        .unwrap();
    let wrong = registry
        .register_local(&root, key("wrong"), CObjectType::scalar(T::U64))
        .unwrap();
    let identity = registry.register_loop(&root, key("loop_identity")).unwrap();
    let ast = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function).unwrap();
    let counter_place = ast.local(counter.clone()).unwrap();
    let counter_read = ast.read(counter_place.clone()).unwrap();
    let bound_read = ast.read(ast.local(bound.clone()).unwrap()).unwrap();
    let condition = ast
        .numeric_conversion(
            T::Bool,
            ast.binary(CBinaryOperator::Less, counter_read.clone(), bound_read)
                .unwrap(),
        )
        .unwrap();
    let one = ast
        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(1)))
        .unwrap();
    let update = statements
        .assign(
            counter_place,
            ast.binary(CBinaryOperator::Add, counter_read, one).unwrap(),
        )
        .unwrap();
    let body = statements
        .block(
            body_scope,
            vec![
                update,
                statements.continue_statement(identity.clone()).unwrap(),
            ],
        )
        .unwrap();
    let loop_statement = statements
        .counted_loop(
            identity.clone(),
            counter.clone(),
            bound.clone(),
            condition.clone(),
            body.clone(),
        )
        .unwrap();
    assert_eq!(
        statements
            .continue_statement(identity.clone())
            .unwrap()
            .kind(),
        &CStatementKind::Continue(identity.clone())
    );
    let CStatementKind::BoundedLoop {
        identity: actual_identity,
        progress,
        condition: actual,
        body: actual_body,
        ..
    } = loop_statement.kind()
    else {
        panic!("counted loop")
    };
    assert_eq!(progress.counter(), &counter);
    assert_eq!(actual_identity, &identity);
    assert_eq!(progress.bound(), &bound);
    assert_eq!(progress.step(), CCountedStep::One);
    assert_eq!(actual, &condition);
    assert_eq!(actual_body, &body);
    assert_eq!(
        statements.counted_loop(identity, wrong, bound.clone(), condition, body),
        Err(E::InvalidCountedProgress)
    );
    let zero = ast
        .expression_initializer(
            ast.literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
                .unwrap(),
        )
        .unwrap();
    let three = ast
        .expression_initializer(
            ast.literal(CLiteral::Unsigned(CUnsignedLiteral::Size(3)))
                .unwrap(),
        )
        .unwrap();
    assert!(
        statements
            .block(
                root,
                vec![
                    statements.declare(counter, Some(zero)).unwrap(),
                    statements.declare(bound, Some(three)).unwrap(),
                    loop_statement
                ]
            )
            .is_ok()
    );
}

#[test]
fn switch_cleanup_and_break_nodes_retain_owners_and_reject_wrong_categories() {
    let (mut registry, function, root) = setup(CReturnType::Void);
    let arm_scope = registry
        .register_scope(&function, Some(&root), key("arm"))
        .unwrap();
    let default_scope = registry
        .register_scope(&function, Some(&root), key("default_arm"))
        .unwrap();
    let switch = registry
        .register_switch(&root, key("switch_identity"))
        .unwrap();
    let cleanup = registry
        .register_cleanup_exit(&root, key("cleanup"))
        .unwrap();
    let local = registry
        .register_local(&root, key("local"), CObjectType::scalar(T::I32))
        .unwrap();
    let floating = registry
        .register_object(
            function.file(),
            key("floating"),
            CObjectType::scalar(T::F64),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function).unwrap();
    let arm_body = statements
        .block(
            arm_scope,
            vec![
                statements
                    .break_statement(CBreakTarget::Switch(switch.clone()))
                    .unwrap(),
            ],
        )
        .unwrap();
    assert_eq!(
        statements.switch_arm(vec![], arm_body.clone()),
        Err(E::EmptyCaseList)
    );
    let arm = statements
        .switch_arm(
            vec![CCaseConstant::Signed(CSignedLiteral::Int(1))],
            arm_body,
        )
        .unwrap();
    let default = statements
        .block(
            default_scope,
            vec![statements.cleanup_jump(cleanup.clone()).unwrap()],
        )
        .unwrap();
    assert_eq!(
        statements.switch_statement(
            switch.clone(),
            ast.read(ast.global(floating).unwrap()).unwrap(),
            vec![arm.clone()],
            default.clone()
        ),
        Err(E::ExpectedIntegerSwitch)
    );
    let value = ast
        .literal(CLiteral::Signed(CSignedLiteral::Int(1)))
        .unwrap();
    let switch = statements
        .switch_statement(switch, value, vec![arm], default)
        .unwrap();
    assert_eq!(
        statements.label(cleanup.clone(), statements.declare(local, None).unwrap()),
        Err(E::LabelBeforeDeclaration)
    );
    let label = statements
        .label(cleanup, statements.return_statement(None).unwrap())
        .unwrap();
    assert!(statements.block(root, vec![switch, label]).is_ok());
}
