use super::{
    DiagnosticCode, JavaBlock, JavaDialect, JavaExpr, JavaIdentifier, JavaKnownType, JavaPrimitive,
    JavaStmt, JavaType, fixture_declaration, parameter, structural_method, verify_fixture,
};

#[test]
fn foreach_binding_matches_list_or_array_element_type() {
    let integer = JavaType::Boxed(JavaPrimitive::Int);
    let list = JavaType::generic(JavaKnownType::List, vec![integer.clone()]);
    let loop_statement = |binding_type| JavaStmt::ForEach {
        binding_type,
        binding: JavaIdentifier::from_portable("item"),
        iterable: JavaExpr::local(list.clone(), JavaIdentifier::from_portable("values")),
        body: JavaBlock::new(vec![]),
    };
    let invalid = fixture_declaration(vec![structural_method(
        "visit",
        JavaType::primitive(JavaPrimitive::Void),
        vec![parameter(list.clone(), "values")],
        JavaBlock::new(vec![loop_statement(JavaType::known(JavaKnownType::String))]),
    )]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], invalid)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch
            && value.message.contains("iterable element type")
    }));

    let valid = fixture_declaration(vec![structural_method(
        "visit",
        JavaType::primitive(JavaPrimitive::Void),
        vec![parameter(list.clone(), "values")],
        JavaBlock::new(vec![loop_statement(integer)]),
    )]);
    assert!(
        verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], valid)],
        )
        .is_ok()
    );
}

#[test]
fn java_erasure_rejects_generic_overloads_with_the_same_raw_signature() {
    let method = |argument: JavaType, parameter_name: &str| {
        structural_method(
            "accept",
            JavaType::primitive(JavaPrimitive::Void),
            vec![parameter(argument, parameter_name)],
            JavaBlock::new(vec![]),
        )
    };
    let invalid = fixture_declaration(vec![
        method(
            JavaType::generic(
                JavaKnownType::List,
                vec![JavaType::known(JavaKnownType::String)],
            ),
            "strings",
        ),
        method(
            JavaType::generic(
                JavaKnownType::List,
                vec![JavaType::Boxed(JavaPrimitive::Int)],
            ),
            "integers",
        ),
    ]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], invalid)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::DuplicateDeclaration
            && value.message.contains("erased declaration signature")
    }));

    let valid = fixture_declaration(vec![
        method(
            JavaType::generic(
                JavaKnownType::List,
                vec![JavaType::known(JavaKnownType::String)],
            ),
            "strings",
        ),
        method(JavaType::known(JavaKnownType::String), "string"),
    ]);
    assert!(
        verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], valid)],
        )
        .is_ok()
    );
}
