//! Closed nominal/declaration variants and exact source-group payloads.
use super::{
    declarations::source_file,
    registry_nominals::{key, registry},
};
use crate::ast::{
    CAggregateRef, CAssertDiagnostic, CComment, CDeclarationKind as D, CDeclarations, CExpressions,
    CFileError as E, CFileItem, CFileRole, CFunctionType, CKnownConstant, CLinkage, CObjectType,
    CReturnType, CScalarType, CStatements,
};
#[test]
fn union_and_declaration_payloads_have_exact_inventory_and_placement_controls() {
    let (mut registry, header) = registry();
    let source = source_file(&mut registry, "generated.c", CFileRole::GeneratedSource);
    let union = registry.declare_union(&header, key("Payload")).unwrap();
    let owner = CAggregateRef::Union(union);
    let enumeration = registry.declare_enum(&header, key("Status")).unwrap();
    let incomplete = registry.declare_enum(&header, key("Incomplete")).unwrap();
    let constant = registry
        .register_enumerator(&enumeration, key("Success"), 0)
        .unwrap();
    registry
        .define_enum(&enumeration, vec![constant.clone()])
        .unwrap();
    let member = registry
        .register_member(&owner, key("value"), CObjectType::scalar(CScalarType::I32))
        .unwrap();
    registry
        .define_aggregate(&owner, vec![member.clone()])
        .unwrap();
    let alias = registry
        .register_typedef(
            &header,
            key("Number"),
            CObjectType::scalar(CScalarType::I32),
        )
        .unwrap();
    let function = registry
        .register_function(
            &header,
            key("run"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let object = registry
        .register_object(
            &header,
            key("object"),
            CObjectType::scalar(CScalarType::I32),
        )
        .unwrap();
    let declarations = CDeclarations::new(&registry, header.clone()).unwrap();
    let cases = [
        (
            declarations.forward_tag(owner.clone()).unwrap(),
            D::ForwardTag(owner.clone()),
        ),
        (
            declarations.aggregate(owner.clone()).unwrap(),
            D::Aggregate {
                owner,
                members: vec![member],
            },
        ),
        (
            declarations.typedef(alias.clone()).unwrap(),
            D::Typedef(alias),
        ),
        (
            declarations.enumeration(enumeration.clone()).unwrap(),
            D::Enum {
                owner: enumeration.clone(),
                values: vec![constant],
            },
        ),
        (
            declarations
                .function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
            D::FunctionPrototype {
                function: function.clone(),
                linkage: CLinkage::External,
            },
        ),
        (
            declarations.object_declaration(object.clone()).unwrap(),
            D::ObjectDeclaration(object.clone()),
        ),
    ];
    for (declaration, kind) in cases {
        assert_eq!(declaration.file(), &header);
        assert_eq!(declaration.kind(), &kind);
    }
    assert_eq!(
        declarations.enumeration(incomplete),
        Err(E::IncompleteDefinition)
    );
    let other = CDeclarations::new(&registry, source).unwrap();
    assert_eq!(other.enumeration(enumeration), Err(E::WrongFile));
    assert_eq!(
        other.function_prototype(function, CLinkage::External),
        Err(E::WrongFile)
    );
    assert_eq!(other.object_declaration(object), Err(E::WrongFile));
}
#[test]
fn files_retain_all_items_in_order_and_reject_misplaced_definitions() {
    let (mut registry, header) = registry();
    let source = source_file(&mut registry, "generated.c", CFileRole::GeneratedSource);
    let function = registry
        .register_function(
            &header,
            key("run"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("root"))
        .unwrap();
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let body = statements
        .block(scope, vec![statements.return_statement(None).unwrap()])
        .unwrap();
    let declarations = CDeclarations::new(&registry, source.clone()).unwrap();
    let definition = declarations
        .function_definition(function, CLinkage::External, vec![], body)
        .unwrap();
    let condition = CExpressions::new(&registry).known_constant(CKnownConstant::CharBit);
    let diagnostic = CAssertDiagnostic::new(b"width".to_vec());
    let assertion = declarations
        .static_assert(condition.clone(), diagnostic.clone())
        .unwrap();
    assert_eq!(assertion.file(), &source);
    assert_eq!(assertion.condition(), &condition);
    assert_eq!(assertion.diagnostic(), &diagnostic);
    let items = vec![
        CFileItem::Comment(CComment::new("first")),
        CFileItem::Definition(definition.clone()),
        CFileItem::StaticAssert(assertion),
        CFileItem::Comment(CComment::new("last")),
    ];
    let file = declarations.source_file(items.clone()).unwrap();
    assert_eq!(file.identity(), &source);
    assert_eq!(file.items(), items);
    assert_eq!(
        CDeclarations::new(&registry, header)
            .unwrap()
            .source_file(vec![CFileItem::Definition(definition)]),
        Err(E::WrongFile)
    );
}
