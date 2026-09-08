//! Remaining closed control categories retain owners and actual child trees.
use super::{registry_nominals::key, statements::setup};
use crate::ast::{
    CBreakTarget, CCaseConstant, CExpressions, CLiteral, CRegistryError, CReturnType,
    CStatementError as E, CStatementKind as K, CStatements, CUnsignedLiteral,
};
#[test]
fn cases_cover_unsigned_and_authenticated_enumerator_branches() {
    let (mut registry, function, root) = setup(CReturnType::Void);
    let owner = registry
        .declare_enum(function.file(), key("Status"))
        .unwrap();
    let enumerator = registry
        .register_enumerator(&owner, key("Ready"), 1)
        .unwrap();
    registry
        .define_enum(&owner, vec![enumerator.clone()])
        .unwrap();
    let child = registry
        .register_scope(&function, Some(&root), key("arm"))
        .unwrap();
    let other_child = registry
        .register_scope(&function, Some(&root), key("default_body"))
        .unwrap();
    let identity = registry.register_switch(&root, key("selection")).unwrap();
    let (mut foreign, foreign_function, _) = setup(CReturnType::Void);
    let foreign_owner = foreign
        .declare_enum(foreign_function.file(), key("Status"))
        .unwrap();
    let foreign_enumerator = foreign
        .register_enumerator(&foreign_owner, key("Ready"), 1)
        .unwrap();
    let ast = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function).unwrap();
    assert_eq!(
        statements
            .break_statement(CBreakTarget::Switch(identity.clone()))
            .unwrap()
            .kind(),
        &K::Break(CBreakTarget::Switch(identity.clone()))
    );
    let body = statements.block(child, vec![statements.empty()]).unwrap();
    let cases = vec![
        CCaseConstant::Unsigned(CUnsignedLiteral::U64(3)),
        CCaseConstant::Enumerator(enumerator),
    ];
    let arm = statements.switch_arm(cases.clone(), body.clone()).unwrap();
    assert_eq!(arm.cases(), cases);
    assert_eq!(arm.body(), &body);
    assert_eq!(
        statements.switch_arm(vec![CCaseConstant::Enumerator(foreign_enumerator)], body),
        Err(E::Registry(CRegistryError::CrossRegistry))
    );
    let value = ast
        .literal(CLiteral::Unsigned(CUnsignedLiteral::U64(3)))
        .unwrap();
    let default = statements
        .block(other_child, vec![statements.empty()])
        .unwrap();
    let result = statements
        .switch_statement(
            identity.clone(),
            value.clone(),
            vec![arm.clone()],
            default.clone(),
        )
        .unwrap();
    assert_eq!(
        result.kind(),
        &K::Switch {
            identity,
            value,
            arms: vec![arm],
            default,
        }
    );
}
#[test]
fn loop_break_and_nested_block_retain_identity_and_reject_crossed_owners() {
    let (mut registry, function, root) = setup(CReturnType::Void);
    let child = registry
        .register_scope(&function, Some(&root), key("child"))
        .unwrap();
    let sibling = registry
        .register_scope(&function, Some(&root), key("sibling"))
        .unwrap();
    let identity = registry.register_loop(&root, key("loop_identity")).unwrap();
    let other = registry
        .register_function(
            function.file(),
            key("other"),
            crate::ast::CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let other_scope = registry.register_scope(&other, None, key("root")).unwrap();
    let other_loop = registry
        .register_loop(&other_scope, key("other_loop"))
        .unwrap();
    let statements = CStatements::new(&registry, function).unwrap();
    let target = CBreakTarget::Loop(identity);
    let stop = statements.break_statement(target.clone()).unwrap();
    assert_eq!(stop.kind(), &K::Break(target));
    assert_eq!(
        statements.break_statement(CBreakTarget::Loop(other_loop)),
        Err(E::WrongScope)
    );
    let body = statements.block(child, vec![stop]).unwrap();
    let block = statements.nested_block(body.clone()).unwrap();
    assert_eq!(block.kind(), &K::Block(body));
    assert!(statements.block(root, vec![block.clone()]).is_ok());
    assert_eq!(statements.block(sibling, vec![block]), Err(E::WrongScope));
    let foreign_block = CStatements::new(&registry, other)
        .unwrap()
        .block(other_scope, vec![])
        .unwrap();
    assert_eq!(statements.nested_block(foreign_block), Err(E::WrongScope));
}
