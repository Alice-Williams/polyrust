use super::{
    JavaCompileFailField, JavaDeclarationKind, JavaExpr, JavaHeritage, JavaIdentifier,
    JavaKnownType, JavaLiteral, JavaMember, JavaModifier, JavaPrimitive, JavaType,
    JavaTypeDeclaration, JavaVisibility,
};

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
