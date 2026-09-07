use super::{
    DiagnosticCode, GeneratedSymbolId, GeneratedValueId, JavaBlock, JavaCallableRef, JavaCatch,
    JavaConstructor, JavaDialect, JavaExpr, JavaExprKind, JavaField, JavaIdentifier, JavaKnownType,
    JavaLiteral, JavaMember, JavaMemberOrigin, JavaModifier, JavaParameter, JavaPrecedence,
    JavaPrimitive, JavaStmt, JavaType, JavaValueRef, TargetTypeRef, fixture_declaration, parameter,
    structural_method, this_field, verifier_source, verify_fixture,
};

#[test]
fn generated_field_initializers_reject_self_and_forward_references() {
    fn generated_value(
        builder: &mut portable_codegen::TargetAstBuilder<JavaDialect>,
        name: &str,
    ) -> GeneratedValueId {
        builder.value(portable_codegen::GeneratedValue {
            name: name.to_owned(),
            ty: TargetTypeRef::Primitive(JavaPrimitive::Int),
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source(name),
        })
    }

    fn field(id: GeneratedValueId, name: &str, initializer: JavaExpr) -> JavaMember {
        JavaMember::Field(JavaField {
            declared: Some(id),
            modifiers: vec![JavaModifier::Static, JavaModifier::Final],
            ty: JavaType::primitive(JavaPrimitive::Int),
            name: JavaIdentifier::from_portable(name),
            initializer: Some(initializer),
        })
    }

    fn reference(id: GeneratedValueId) -> JavaExpr {
        JavaExpr {
            ty: JavaType::primitive(JavaPrimitive::Int),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(id))),
        }
    }

    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let dependent = generated_value(&mut builder, "dependent");
    let dependency = generated_value(&mut builder, "dependency");
    let declaration = fixture_declaration(vec![
        field(dependent, "dependent", reference(dependency)),
        field(
            dependency,
            "dependency",
            JavaExpr::literal(JavaType::primitive(JavaPrimitive::Int), JavaLiteral::I32(7)),
        ),
    ]);
    let diagnostics = verify_fixture(
        builder,
        vec![(
            vec![
                GeneratedSymbolId::Value(dependent),
                GeneratedSymbolId::Value(dependency),
            ],
            declaration,
        )],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("itself or a later field")
    }));

    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let recursive = generated_value(&mut builder, "recursive");
    let declaration =
        fixture_declaration(vec![field(recursive, "recursive", reference(recursive))]);
    let diagnostics = verify_fixture(
        builder,
        vec![(vec![GeneratedSymbolId::Value(recursive)], declaration)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("itself or a later field")
    }));
}

fn decoder_decode_call() -> (JavaExpr, Vec<JavaParameter>) {
    let decoder = JavaType::known(JavaKnownType::CharsetDecoder);
    let buffer = JavaType::known(JavaKnownType::ByteBuffer);
    let signature = crate::dialect::JavaKnownMethod::DecoderDecode.signature();
    (
        JavaExpr {
            ty: signature.result.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Call {
                callable: JavaCallableRef::Member {
                    owner: decoder.clone(),
                    name: JavaIdentifier::from_portable("decode"),
                    signature,
                    origin: JavaMemberOrigin::Known(crate::dialect::JavaKnownMethod::DecoderDecode),
                },
                receiver: Some(Box::new(JavaExpr::local(
                    decoder.clone(),
                    JavaIdentifier::from_portable("decoder"),
                ))),
                arguments: vec![JavaExpr::local(
                    buffer.clone(),
                    JavaIdentifier::from_portable("buffer"),
                )],
            },
        },
        vec![parameter(decoder, "decoder"), parameter(buffer, "buffer")],
    )
}

#[test]
fn checked_calls_require_a_matching_catch_when_throws_are_not_modelled() {
    let (call, parameters) = decoder_decode_call();
    let result = call.ty.clone();
    let uncaught = fixture_declaration(vec![structural_method(
        "decode",
        result.clone(),
        parameters.clone(),
        JavaBlock::new(vec![JavaStmt::Return(Some(call))]),
    )]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], uncaught)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidInvocation
            && value.message.contains("unhandled checked exceptions")
    }));

    let (call, parameters) = decoder_decode_call();
    let caught = fixture_declaration(vec![structural_method(
        "decode",
        result.clone(),
        parameters,
        JavaBlock::new(vec![JavaStmt::TryCatch {
            try_block: JavaBlock::new(vec![JavaStmt::Return(Some(call))]),
            catches: vec![JavaCatch {
                exception_type: JavaType::known(JavaKnownType::CharacterCodingException),
                binding: JavaIdentifier::from_portable("failure"),
                body: JavaBlock::new(vec![JavaStmt::ThrowAssertion(JavaExpr::literal(
                    JavaType::known(JavaKnownType::String),
                    JavaLiteral::String("decode failed".to_owned()),
                ))]),
            }],
        }]),
    )]);
    let verification = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], caught)],
    );
    assert!(verification.is_ok(), "{verification:?}");
}

#[test]
fn field_initializers_have_lexical_blank_final_and_exception_preflight() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let unresolved = fixture_declaration(vec![JavaMember::Field(JavaField {
        declared: None,
        modifiers: vec![JavaModifier::Static, JavaModifier::Final],
        ty: int.clone(),
        name: JavaIdentifier::from_portable("unresolved"),
        initializer: Some(JavaExpr::local(
            int.clone(),
            JavaIdentifier::from_portable("missing"),
        )),
    })]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], unresolved)],
    )
    .unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|value| value.code == DiagnosticCode::UnresolvedReference)
    );

    let (checked_call, _) = decoder_decode_call();
    let checked = fixture_declaration(vec![JavaMember::Field(JavaField {
        declared: None,
        modifiers: vec![JavaModifier::Static, JavaModifier::Final],
        ty: checked_call.ty.clone(),
        name: JavaIdentifier::from_portable("checked"),
        initializer: Some(checked_call),
    })]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], checked)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidInvocation
            && value.message.contains("field initializer")
    }));

    let owner = JavaType::known(JavaKnownType::RuntimeError);
    let assignment = JavaStmt::Assign {
        target: this_field(owner.clone(), int.clone(), "x"),
        value: JavaExpr::literal(int.clone(), JavaLiteral::I32(1)),
    };
    let mut blank_read = fixture_declaration(vec![
        JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Private, JavaModifier::Final],
            ty: int.clone(),
            name: JavaIdentifier::from_portable("x"),
            initializer: None,
        }),
        JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Private, JavaModifier::Final],
            ty: int.clone(),
            name: JavaIdentifier::from_portable("y"),
            initializer: Some(this_field(owner, int, "x")),
        }),
        JavaMember::Constructor(JavaConstructor {
            modifiers: vec![],
            name: JavaIdentifier::from_portable("PolyError"),
            parameters: vec![],
            body: JavaBlock::new(vec![assignment]),
        }),
    ]);
    blank_read.name = JavaIdentifier::from_portable("PolyError");
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], blank_read)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value
                .message
                .contains("field initializer reads blank final")
    }));
}
