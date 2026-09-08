use super::{
    JavaCompileFailField, JavaDeclarationKind, JavaExpr, JavaHeritage, JavaIdentifier,
    JavaKnownType, JavaLiteral, JavaMember, JavaModifier, JavaPrimitive, JavaType,
    JavaTypeDeclaration, JavaVisibility,
};

#[test]
fn compile_fail_fields_require_a_closed_invalid_shape() {
    let string = || {
        JavaExpr::literal(
            JavaType::known(JavaKnownType::String),
            JavaLiteral::String("missing".to_owned()),
        )
    };
    let integer =
        || JavaExpr::literal(JavaType::primitive(JavaPrimitive::Int), JavaLiteral::I32(1));
    for (expected_type, initializer, accepted) in [
        (JavaType::primitive(JavaPrimitive::Int), string(), true),
        (JavaType::primitive(JavaPrimitive::Long), integer(), false),
        (JavaType::primitive(JavaPrimitive::Byte), integer(), false),
        (JavaType::known(JavaKnownType::Object), string(), false),
        (JavaType::primitive(JavaPrimitive::Int), integer(), false),
    ] {
        let declaration =
            super::fixture_declaration(vec![JavaMember::CompileFailField(JavaCompileFailField {
                modifiers: vec![JavaModifier::Final],
                expected_type,
                name: JavaIdentifier::from_portable("invalid"),
                initializer,
            })]);
        let result = super::verify_file_items(
            portable_codegen::TargetAstBuilder::new(crate::dialect::JavaDialect),
            portable_codegen::SourceRole::NegativeTest,
            crate::ast::JavaFilePlacement::NegativeTest,
            vec![crate::ast::JavaFileItem::Type {
                conformances: crate::ast::JavaConformanceInventory::structural().into(),
                declared: vec![],
                declaration,
            }],
        );
        assert_eq!(result.is_ok(), accepted, "{result:?}");
    }
}

#[test]
fn compile_fail_members_are_explicit_and_discoverable() {
    let declaration = JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("InvalidTypes"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::CompileFailField(JavaCompileFailField {
            modifiers: vec![JavaModifier::Final],
            expected_type: JavaType::generic(
                JavaKnownType::RuntimeOption,
                vec![JavaType::Boxed(JavaPrimitive::Int)],
            ),
            name: JavaIdentifier::from_portable("invalid"),
            initializer: JavaExpr::literal(
                JavaType::known(JavaKnownType::String),
                JavaLiteral::String("missing".to_owned()),
            ),
        })],
    };
    assert!(declaration.contains_compile_fail_member());
}
