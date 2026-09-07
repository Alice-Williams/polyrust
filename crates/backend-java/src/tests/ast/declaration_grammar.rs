use super::{
    DiagnosticCode, JavaBlock, JavaConstructor, JavaDeclarationKind, JavaDialect, JavaExpr,
    JavaField, JavaHeritage, JavaIdentifier, JavaKnownType, JavaLiteral, JavaMember, JavaMethod,
    JavaMethodDeclaration, JavaModifier, JavaPrimitive, JavaRecordComponent,
    JavaRecordComponentOrigin, JavaStmt, JavaType, JavaTypeDeclaration, JavaVisibility,
    fixture_core_field, fixture_declaration, parameter, verify_fixture,
};

#[test]
fn declaration_kinds_enforce_interface_members_and_canonical_records() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let interface = JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("InvalidInterface"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Static, JavaModifier::Final],
            ty: int.clone(),
            name: JavaIdentifier::from_portable("value"),
            initializer: Some(JavaExpr::literal(int.clone(), JavaLiteral::I32(1))),
        })],
    };
    let record = JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("InvalidRecord"),
        type_parameters: vec![],
        record_components: vec![JavaRecordComponent {
            origin: JavaRecordComponentOrigin::Core(fixture_core_field()),
            ty: int,
            name: JavaIdentifier::from_portable("value"),
        }],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Constructor(JavaConstructor {
            modifiers: vec![],
            name: JavaIdentifier::from_portable("InvalidRecord"),
            parameters: vec![parameter(JavaType::known(JavaKnownType::String), "wrong")],
            body: JavaBlock::new(vec![]),
        })],
    };
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], interface), (vec![], record)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidStructure
            && value.message.contains("interfaces cannot declare fields")
    }));
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidStructure
            && value.message.contains("canonical component signature")
    }));
}

#[test]
fn record_object_and_nested_type_grammar_fail_closed() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let component = |name: &str| JavaRecordComponent {
        origin: JavaRecordComponentOrigin::Core(fixture_core_field()),
        ty: int.clone(),
        name: JavaIdentifier::from_portable(name),
    };

    let reserved_component = JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("Fixture"),
        type_parameters: vec![],
        record_components: vec![component("hashCode")],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![],
    };
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], reserved_component)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("reserved Object member")
    }));

    let inaccessible_constructor = JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Public,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("Fixture"),
        type_parameters: vec![],
        record_components: vec![component("x")],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Private],
            name: JavaIdentifier::from_portable("Fixture"),
            parameters: vec![parameter(int.clone(), "x")],
            body: JavaBlock::new(vec![]),
        })],
    };
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], inaccessible_constructor)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("less accessible")
    }));

    let invalid_accessor = JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("Fixture"),
        type_parameters: vec![],
        record_components: vec![component("x")],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![],
            modifiers: vec![JavaModifier::Private],
            type_parameters: vec![],
            return_type: int.clone(),
            name: JavaIdentifier::from_portable("x"),
            parameters: vec![],
            body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
                JavaExpr::literal(int.clone(), JavaLiteral::I32(1)),
            ))])),
        })],
    };
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], invalid_accessor)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("record accessor")
    }));

    let inherited_collision = fixture_declaration(vec![JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Private],
        type_parameters: vec![],
        return_type: JavaType::primitive(JavaPrimitive::Boolean),
        name: JavaIdentifier::from_portable("equals"),
        parameters: vec![parameter(JavaType::known(JavaKnownType::Object), "other")],
        body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
            JavaExpr::literal(
                JavaType::primitive(JavaPrimitive::Boolean),
                JavaLiteral::Boolean(true),
            ),
        ))])),
    })]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], inherited_collision)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("inherited Object method")
    }));

    let nested_collision = fixture_declaration(vec![JavaMember::NestedType(JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Private,
        modifiers: vec![JavaModifier::Static],
        name: JavaIdentifier::from_portable("Fixture"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![],
    })]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], nested_collision)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::DuplicateDeclaration
            && diagnostic.message.contains("enclosing type name")
    }));
}
