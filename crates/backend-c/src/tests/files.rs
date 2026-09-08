//! Safe comments and exact unresolved file grouping; no includes/raw source API.

use super::{
    declarations::source_file,
    registry_nominals::{key, registry},
};
use crate::ast::{
    CAssertDiagnostic, CComment, CDeclarations, CExpressions, CFileError as E, CFileItem,
    CFileRole, CKnownConstant, CLiteral, CObjectType, CScalarType as T, CSignedLiteral,
};

#[test]
fn comments_break_translation_phase_escape_sequences_and_preserve_line_structure() {
    let comment = CComment::new("first\r\nsecond\rthird\n*/ #include \"evil\"\n??/\\\n\0é");
    assert!(!comment.text().contains("*/"));
    assert!(!comment.text().contains("??"));
    assert!(!comment.text().contains('\\'));
    assert!(!comment.text().contains('\r'));
    assert!(!comment.text().contains('\0'));
    assert!(comment.text().starts_with("first\nsecond\nthird\n"));
    assert!(comment.text().is_ascii());
    assert!(comment.text().contains("[0x00][0xC3][0xA9]"));
    assert_eq!(CComment::new("ordinary text").text(), "ordinary text");
    let diagnostic = CAssertDiagnostic::new(b"\0\"\\??/".to_vec());
    assert_eq!(diagnostic.bytes(), b"\0\"\\??/");
}

#[test]
fn static_assertions_require_integer_constant_shape_not_runtime_reads_or_float_computation() {
    let (mut registry, header) = registry();
    let global = registry
        .register_object(&header, key("global"), CObjectType::scalar(T::Int))
        .unwrap();
    let ast = CExpressions::new(&registry);
    let declarations = CDeclarations::new(&registry, header).unwrap();
    let message = CAssertDiagnostic::new(b"platform".to_vec());
    let constant = ast.known_constant(CKnownConstant::CharBit);
    assert!(
        declarations
            .static_assert(constant, message.clone())
            .is_ok()
    );
    assert!(
        declarations
            .static_assert(
                ast.size_of(CObjectType::scalar(T::F64)).unwrap(),
                message.clone()
            )
            .is_ok()
    );
    assert_eq!(
        declarations.static_assert(
            ast.read(ast.global(global).unwrap()).unwrap(),
            message.clone()
        ),
        Err(E::ExpectedIntegerConstantExpression)
    );
    let one = ast
        .literal(CLiteral::Signed(CSignedLiteral::Int(1)))
        .unwrap();
    let floating = ast.numeric_conversion(T::F64, one).unwrap();
    assert_eq!(
        declarations.static_assert(floating.clone(), message.clone()),
        Err(E::ExpectedIntegerConstantExpression)
    );
    assert_eq!(
        declarations.static_assert(ast.numeric_conversion(T::Int, floating).unwrap(), message),
        Err(E::ExpectedIntegerConstantExpression)
    );
}

#[test]
fn files_accept_only_items_owned_by_the_exact_registered_group() {
    let (mut registry, header) = registry();
    let source = source_file(&mut registry, "src/generated.c", CFileRole::GeneratedSource);
    let alias = registry
        .register_typedef(&header, key("Value"), CObjectType::scalar(T::I32))
        .unwrap();
    let declarations = CDeclarations::new(&registry, header.clone()).unwrap();
    let item = CFileItem::Declaration(declarations.typedef(alias).unwrap());
    let items = vec![CFileItem::Comment(CComment::new("Generated")), item.clone()];
    let file = declarations.source_file(items.clone()).unwrap();
    assert_eq!(file.items(), items);
    assert_eq!(file.identity(), &header);
    assert_eq!(file.items().len(), 2);
    let assertion = declarations
        .static_assert(
            CExpressions::new(&registry).known_constant(CKnownConstant::CharBit),
            CAssertDiagnostic::new(b"char width".to_vec()),
        )
        .unwrap();
    assert!(
        declarations
            .source_file(vec![CFileItem::StaticAssert(assertion.clone())])
            .is_ok()
    );
    assert_eq!(
        CDeclarations::new(&registry, source.clone())
            .unwrap()
            .source_file(vec![CFileItem::StaticAssert(assertion)]),
        Err(E::WrongFile)
    );
    assert_eq!(
        CDeclarations::new(&registry, source)
            .unwrap()
            .source_file(vec![item]),
        Err(E::WrongFile)
    );
}
