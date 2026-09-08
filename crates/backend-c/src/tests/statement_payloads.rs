//! Ordinary statements preserve identities and complete ordered children.
use super::{registry_nominals::key, statements::setup};
use crate::ast::{
    CExpressionError, CExpressions, CFunctionType, CLiteral, CObjectType, CRegistryError,
    CReturnType, CReturnValue, CScalarType, CSignedLiteral, CStatementError as E,
    CStatementKind as K, CStatements,
};
#[test]
fn ordinary_statements_preserve_every_child_and_function_owner() {
    let scalar = CObjectType::scalar(CScalarType::I32);
    let (mut registry, function, root) = setup(CReturnType::Value(
        CReturnValue::new(scalar.clone()).unwrap(),
    ));
    let local = registry
        .register_local(&root, key("local"), scalar)
        .unwrap();
    let then_scope = registry
        .register_scope(&function, Some(&root), key("then_body"))
        .unwrap();
    let else_scope = registry
        .register_scope(&function, Some(&root), key("else_body"))
        .unwrap();
    let cleanup = registry
        .register_cleanup_exit(&root, key("cleanup"))
        .unwrap();
    let callback = registry
        .register_function(
            function.file(),
            key("callback"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let value = ast
        .literal(CLiteral::Signed(CSignedLiteral::I32(4)))
        .unwrap();
    let initializer = ast.expression_initializer(value.clone()).unwrap();
    let declaration = statements
        .declare(local.clone(), Some(initializer.clone()))
        .unwrap();
    let K::Declare(actual) = declaration.kind() else {
        panic!("declaration")
    };
    assert_eq!(actual.local(), &local);
    assert_eq!(actual.initializer(), Some(&initializer));
    let uninitialized = statements.declare(local.clone(), None).unwrap();
    let K::Declare(actual) = uninitialized.kind() else {
        panic!("declaration")
    };
    assert_eq!(actual.local(), &local);
    assert_eq!(actual.initializer(), None);
    assert_eq!(declaration.function(), &function);
    assert_eq!(
        CStatements::new(&registry, callback.clone())
            .unwrap()
            .return_statement(None)
            .unwrap()
            .kind(),
        &K::Return(None)
    );
    let effect = ast
        .call_effect(ast.direct(callback).unwrap(), vec![])
        .unwrap();
    let evaluated = statements.evaluate(effect.clone()).unwrap();
    assert_eq!(evaluated.kind(), &K::Evaluate(effect));
    let discarded = statements.discard(value.clone()).unwrap();
    assert_eq!(discarded.kind(), &K::Discard(value.clone()));
    let returned = statements.return_statement(Some(value.clone())).unwrap();
    assert_eq!(returned.kind(), &K::Return(Some(value)));
    let empty = statements.empty();
    assert_eq!(empty.kind(), &K::Empty);
    let then_body = statements
        .block(then_scope, vec![discarded.clone()])
        .unwrap();
    let else_body = statements.block(else_scope, vec![empty.clone()]).unwrap();
    let condition = ast.literal(CLiteral::Bool(true)).unwrap();
    let branch = statements
        .if_statement(condition.clone(), then_body.clone(), else_body.clone())
        .unwrap();
    assert_eq!(
        branch.kind(),
        &K::If {
            condition,
            then_block: then_body.clone(),
            else_block: else_body.clone()
        }
    );
    assert_eq!(
        statements.if_statement(
            ast.literal(CLiteral::Signed(CSignedLiteral::I32(1)))
                .unwrap(),
            then_body,
            else_body
        ),
        Err(E::Expression(CExpressionError::ExpectedBool))
    );
    let jump = statements.cleanup_jump(cleanup.clone()).unwrap();
    assert_eq!(jump.kind(), &K::CleanupJump(cleanup.clone()));
    let label = statements.label(cleanup.clone(), returned.clone()).unwrap();
    assert_eq!(
        label.kind(),
        &K::Label {
            identity: cleanup,
            statement: Box::new(returned)
        }
    );
    let children = vec![
        declaration,
        evaluated,
        discarded,
        empty,
        branch,
        jump,
        label,
    ];
    let block = statements.block(root.clone(), children.clone()).unwrap();
    assert_eq!(block.scope(), &root);
    assert_eq!(block.statements(), children);
    let (foreign_registry, foreign_function, _) = setup(CReturnType::Void);
    let foreign = CExpressions::new(&foreign_registry);
    let value = foreign.literal(CLiteral::Bool(false)).unwrap();
    assert_eq!(
        statements.discard(value),
        Err(E::Expression(CExpressionError::Registry(
            CRegistryError::CrossRegistry
        )))
    );
    let effect = foreign
        .call_effect(foreign.direct(foreign_function).unwrap(), vec![])
        .unwrap();
    assert_eq!(
        statements.evaluate(effect),
        Err(E::Expression(CExpressionError::Registry(
            CRegistryError::CrossRegistry
        )))
    );
}
