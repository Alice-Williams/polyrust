use super::{
    DiagnosticCode, JavaBlock, JavaDialect, JavaExpr, JavaField, JavaFilePlacement, JavaIdentifier,
    JavaKnownType, JavaLiteral, JavaMember, JavaModifier, JavaNullPurpose, JavaRuntimeHelper,
    JavaStmt, JavaType, fixture_declaration, structural_method, verify_file_items, verify_fixture,
};

#[test]
fn privileged_literals_fail_closed_outside_registered_runtime_storage() {
    let string = JavaType::known(JavaKnownType::String);
    let declaration = fixture_declaration(vec![
        JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Static, JavaModifier::Final],
            ty: string.clone(),
            name: JavaIdentifier::from_portable("rawSurrogate"),
            initializer: Some(JavaExpr::literal(
                string.clone(),
                JavaLiteral::Utf16Units(vec![0xd800]),
            )),
        }),
        structural_method(
            "leakNull",
            string.clone(),
            vec![],
            JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
                string,
                JavaLiteral::InternalNull(JavaNullPurpose::AbsentTaggedPayload),
            )))]),
        ),
    ]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], declaration)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidStructure
            && value.message.contains("raw UTF-16-unit literal")
    }));
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidStructure
            && value
                .message
                .contains("outside exact registered tagged runtime storage")
    }));
}

#[test]
fn registered_runtime_storage_is_valid_but_source_injection_is_rejected() {
    let mut items = vec![crate::runtime::shell_item()];
    items.extend(
        [JavaRuntimeHelper::Core, JavaRuntimeHelper::TaggedValues]
            .into_iter()
            .flat_map(crate::runtime::helper_items),
    );
    let diagnostics = verify_file_items(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        portable_codegen::SourceRole::Runtime,
        JavaFilePlacement::Runtime,
        items,
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("typed helper linker")
    }));
    assert!(!diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("raw UTF-16-unit literal")
            || diagnostic
                .message
                .contains("outside exact registered tagged runtime storage")
    }));
}
