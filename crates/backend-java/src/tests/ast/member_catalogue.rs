use super::{
    DiagnosticCode, GeneratedSymbolId, JavaDialect, JavaExpr, JavaExprKind, JavaField,
    JavaIdentifier, JavaKnownType, JavaMember, JavaMethodSignature, JavaModifier, JavaPrecedence,
    JavaPrimitive, JavaRuntimeMember, JavaType, JavaValueRef, TargetTypeRef, fixture_declaration,
    verifier_source, verify_fixture,
};

#[test]
fn runtime_member_catalogue_rejects_false_owner_and_result_claims() {
    assert_eq!(JavaRuntimeMember::ALL.len(), 14);
    let valid = JavaMethodSignature {
        receiver: Some(JavaType::known(JavaKnownType::RuntimeScalar)),
        parameters: vec![],
        result: JavaType::primitive(JavaPrimitive::Int),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };
    assert!(JavaRuntimeMember::ScalarValue.accepts(&valid));
    let mut wrong = valid.clone();
    wrong.result = JavaType::primitive(JavaPrimitive::Long);
    assert!(!JavaRuntimeMember::ScalarValue.accepts(&wrong));
    wrong = valid;
    wrong.receiver = Some(JavaType::known(JavaKnownType::RuntimeError));
    assert!(!JavaRuntimeMember::ScalarValue.accepts(&wrong));
}

#[test]
fn value_references_match_authoritative_registered_and_known_field_types() {
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let generated = builder.value(portable_codegen::GeneratedValue {
        name: "number".to_owned(),
        ty: TargetTypeRef::Primitive(JavaPrimitive::Int),
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::TestHarness,
        ),
        source: verifier_source("value"),
    });
    let string = JavaType::known(JavaKnownType::String);
    let forged_generated = JavaExpr {
        ty: string.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(generated))),
    };
    let declaration = fixture_declaration(vec![JavaMember::Field(JavaField {
        declared: Some(generated),
        modifiers: vec![JavaModifier::Static, JavaModifier::Final],
        ty: string.clone(),
        name: JavaIdentifier::from_portable("forged"),
        initializer: Some(forged_generated),
    })]);
    let diagnostics = verify_fixture(
        builder,
        vec![(vec![GeneratedSymbolId::Value(generated)], declaration)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch
            && value.message.contains("authoritative registration")
    }));

    let builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let forged_known = JavaExpr {
        ty: string.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::KnownField(
            crate::dialect::JavaKnownField::IntegerMaxValue,
        )),
    };
    let declaration = fixture_declaration(vec![JavaMember::Field(JavaField {
        declared: None,
        modifiers: vec![JavaModifier::Static, JavaModifier::Final],
        ty: string,
        name: JavaIdentifier::from_portable("forged"),
        initializer: Some(forged_known),
    })]);
    let diagnostics = verify_fixture(builder, vec![(vec![], declaration)]).unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch && value.message.contains("catalogue entry")
    }));

    let builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let valid_known = JavaExpr {
        ty: int.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::KnownField(
            crate::dialect::JavaKnownField::IntegerMaxValue,
        )),
    };
    let declaration = fixture_declaration(vec![JavaMember::Field(JavaField {
        declared: None,
        modifiers: vec![JavaModifier::Static, JavaModifier::Final],
        ty: int,
        name: JavaIdentifier::from_portable("valid"),
        initializer: Some(valid_known),
    })]);
    assert!(verify_fixture(builder, vec![(vec![], declaration)]).is_ok());
}
