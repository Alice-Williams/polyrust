//! Statement category, exact return and const-storage controls.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CAggregateRef, CConstness, CExpressions, CFunctionRef, CFunctionType, CLiteral, CObjectType,
    CPointerTarget, CRegistry, CReturnType, CReturnValue, CScalarType as T, CScopeRef,
    CSignedLiteral, CStatementError as E, CStatements,
};

pub(super) fn setup(result: CReturnType) -> (CRegistry, CFunctionRef, CScopeRef) {
    let (mut registry, file) = registry();
    let function = registry
        .register_function(&file, key("run"), CFunctionType::new(result, vec![]))
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("body"))
        .unwrap();
    (registry, function, scope)
}
fn scalar() -> CObjectType {
    CObjectType::scalar(T::I32)
}

#[test]
fn const_local_needs_initializer_and_assignments_require_mutable_matching_places() {
    let (mut registry, function, scope) = setup(CReturnType::Void);
    let mutable = registry
        .register_local(&scope, key("mutable"), scalar())
        .unwrap();
    let constant = registry
        .register_local(
            &scope,
            key("constant"),
            scalar().with_constness(CConstness::Const).unwrap(),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function).unwrap();
    let value = ast
        .literal(CLiteral::Signed(CSignedLiteral::Int(1)))
        .unwrap();
    let initializer = ast.expression_initializer(value.clone()).unwrap();
    assert!(statements.declare(mutable.clone(), None).is_ok());
    assert_eq!(
        statements.declare(constant.clone(), None),
        Err(E::ConstRequiresInitializer)
    );
    let declaration = statements
        .declare(constant.clone(), Some(initializer))
        .unwrap();
    assert!(statements.block(scope, vec![declaration]).is_ok());
    assert!(
        statements
            .assign(ast.local(mutable.clone()).unwrap(), value.clone())
            .is_ok()
    );
    assert_eq!(
        statements.assign(ast.local(constant).unwrap(), value),
        Err(E::NotModifiable)
    );
    assert_eq!(
        statements.assign(
            ast.local(mutable).unwrap(),
            ast.literal(CLiteral::Bool(true)).unwrap()
        ),
        Err(E::TypeMismatch)
    );
}

#[test]
fn const_subobjects_prevent_aggregate_assignment_but_not_pointer_assignment() {
    let (mut registry, function, scope) = setup(CReturnType::Void);
    let file = function.file().clone();
    let inner = registry.declare_struct(&file, key("Inner")).unwrap();
    let inner_owner = CAggregateRef::Struct(inner.clone());
    let constant = registry
        .register_member(
            &inner_owner,
            key("constant"),
            scalar().with_constness(CConstness::Const).unwrap(),
        )
        .unwrap();
    registry
        .define_aggregate(&inner_owner, vec![constant])
        .unwrap();
    let outer = registry.declare_struct(&file, key("Outer")).unwrap();
    let outer_owner = CAggregateRef::Struct(outer.clone());
    let inner_field = registry
        .register_member(&outer_owner, key("inner"), CObjectType::structure(inner))
        .unwrap();
    registry
        .define_aggregate(&outer_owner, vec![inner_field])
        .unwrap();
    let ty = CObjectType::structure(outer);
    let aggregate = registry
        .register_local(&scope, key("aggregate"), ty.clone())
        .unwrap();
    let pointer = registry
        .register_local(
            &scope,
            key("pointer"),
            CObjectType::pointer(CPointerTarget::Object(Box::new(ty))),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function).unwrap();
    assert_eq!(
        statements.declare(aggregate.clone(), None),
        Err(E::ConstRequiresInitializer)
    );
    let place = ast.local(aggregate).unwrap();
    assert_eq!(
        statements.assign(place.clone(), ast.read(place).unwrap()),
        Err(E::NotModifiable)
    );
    let place = ast.local(pointer).unwrap();
    assert!(
        statements
            .assign(place.clone(), ast.read(place).unwrap())
            .is_ok()
    );
}

#[test]
fn return_category_and_explicit_discard_evaluate_are_distinct() {
    let (mut registry, function, scope) =
        setup(CReturnType::Value(CReturnValue::new(scalar()).unwrap()));
    let effect = registry
        .register_function(
            function.file(),
            key("effect"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let value = ast
        .literal(CLiteral::Signed(CSignedLiteral::I32(7)))
        .unwrap();
    assert_eq!(statements.return_statement(None), Err(E::ReturnCategory));
    assert_eq!(
        statements.return_statement(Some(ast.literal(CLiteral::Bool(true)).unwrap())),
        Err(E::TypeMismatch)
    );
    let call = ast
        .call_value(ast.direct(function).unwrap(), vec![])
        .unwrap();
    let discarded = statements.discard(call).unwrap();
    let evaluated = statements
        .evaluate(
            ast.call_effect(ast.direct(effect.clone()).unwrap(), vec![])
                .unwrap(),
        )
        .unwrap();
    assert!(
        statements
            .block(
                scope,
                vec![
                    statements.empty(),
                    discarded,
                    evaluated,
                    statements.return_statement(Some(value.clone())).unwrap()
                ]
            )
            .is_ok()
    );
    let void = CStatements::new(&registry, effect).unwrap();
    assert!(void.return_statement(None).is_ok());
    assert_eq!(void.return_statement(Some(value)), Err(E::ReturnCategory));
}

#[test]
fn block_placement_rejects_sibling_declarations_and_wrong_parent_branches() {
    let (mut registry, function, root) = setup(CReturnType::Void);
    let child = registry
        .register_scope(&function, Some(&root), key("child"))
        .unwrap();
    let sibling = registry
        .register_scope(&function, Some(&root), key("sibling"))
        .unwrap();
    let local = registry
        .register_local(&child, key("value"), scalar())
        .unwrap();
    let ast = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function).unwrap();
    let declaration = statements.declare(local, None).unwrap();
    assert_eq!(
        statements.block(sibling.clone(), vec![declaration.clone()]),
        Err(E::WrongScope)
    );
    let child_block = statements.block(child, vec![declaration]).unwrap();
    let sibling_block = statements.block(sibling.clone(), vec![]).unwrap();
    let branch = statements
        .if_statement(
            ast.literal(CLiteral::Bool(true)).unwrap(),
            child_block,
            sibling_block,
        )
        .unwrap();
    assert_eq!(
        statements.block(sibling, vec![branch.clone()]),
        Err(E::WrongScope)
    );
    assert!(statements.block(root, vec![branch]).is_ok());
}
