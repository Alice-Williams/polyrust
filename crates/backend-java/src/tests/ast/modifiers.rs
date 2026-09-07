use super::{
    DiagnosticCode, JavaAnnotation, JavaBlock, JavaConstructor, JavaDialect, JavaExpr, JavaField,
    JavaHeritage, JavaIdentifier, JavaKnownType, JavaLiteral, JavaMember, JavaMethod,
    JavaMethodDeclaration, JavaModifier, JavaPrimitive, JavaRuntimeMember, JavaStmt, JavaType,
    fixture_declaration, parameter, verify_fixture,
};

#[test]
fn modifier_context_rejects_static_constructors() {
    let invalid = fixture_declaration(vec![JavaMember::Constructor(JavaConstructor {
        modifiers: vec![JavaModifier::Static],
        name: JavaIdentifier::from_portable("Fixture"),
        parameters: vec![],
        body: JavaBlock::new(vec![]),
    })]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], invalid)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidStructure
            && value.message.contains("declaration context")
    }));

    let valid = fixture_declaration(vec![JavaMember::Constructor(JavaConstructor {
        modifiers: vec![JavaModifier::Private],
        name: JavaIdentifier::from_portable("Fixture"),
        parameters: vec![],
        body: JavaBlock::new(vec![]),
    })]);
    assert!(
        verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], valid)],
        )
        .is_ok()
    );
}

#[test]
fn annotations_and_blank_static_finals_fail_closed() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let declaration = fixture_declaration(vec![
        JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![
                JavaModifier::Private,
                JavaModifier::Static,
                JavaModifier::Final,
            ],
            ty: int.clone(),
            name: JavaIdentifier::from_portable("x"),
            initializer: None,
        }),
        JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![JavaAnnotation::Override, JavaAnnotation::Override],
            modifiers: vec![JavaModifier::Static],
            type_parameters: vec![],
            return_type: int.clone(),
            name: JavaIdentifier::from_portable("forgedOverride"),
            parameters: vec![],
            body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
                JavaExpr::literal(int, JavaLiteral::I32(1)),
            ))])),
        }),
        JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![JavaAnnotation::SafeVarargs],
            modifiers: vec![JavaModifier::Static],
            type_parameters: vec![],
            return_type: JavaType::primitive(JavaPrimitive::Void),
            name: JavaIdentifier::from_portable("forgedSafeVarargs"),
            parameters: vec![],
            body: Some(JavaBlock::new(vec![])),
        }),
    ]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], declaration)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidControlFlow
            && diagnostic.message.contains("blank static final")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::DuplicateDeclaration
            && diagnostic.message.contains("annotation")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("no verified instance override")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("varargs parameter form")
    }));

    let weak_method = |member: JavaRuntimeMember| {
        JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![JavaAnnotation::Override],
            modifiers: vec![JavaModifier::Private],
            type_parameters: vec![],
            return_type: JavaType::primitive(JavaPrimitive::Boolean),
            name: JavaIdentifier::from_portable(member.name()),
            parameters: vec![parameter(JavaType::known(JavaKnownType::Object), "other")],
            body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
                JavaExpr::literal(
                    JavaType::primitive(JavaPrimitive::Boolean),
                    JavaLiteral::Boolean(true),
                ),
            ))])),
        })
    };
    let mut weak_runtime_implementation = fixture_declaration(vec![
        weak_method(JavaRuntimeMember::SemanticEquals),
        weak_method(JavaRuntimeMember::DeepEquals),
    ]);
    weak_runtime_implementation.heritage =
        JavaHeritage::Interfaces(vec![JavaType::known(JavaKnownType::RuntimeSemanticValue)]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], weak_runtime_implementation)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("no verified instance override")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InterfaceNonconformance
            && diagnostic.message.contains("semanticEquals")
    }));
}
