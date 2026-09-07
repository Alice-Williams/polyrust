use super::{
    DiagnosticCode, GeneratedSymbolId, GeneratedTypeId, GeneratedValueId, JavaBlock,
    JavaDeclarationKind, JavaDialect, JavaEnumConstant, JavaExpr, JavaHeritage, JavaIdentifier,
    JavaKnownType, JavaLiteral, JavaMember, JavaPattern, JavaPrimitive, JavaStmt, JavaSwitchArm,
    JavaType, JavaTypeDeclaration, JavaTypeName, JavaVisibility, TargetTypeRef,
    fixture_declaration, parameter, structural_method, verifier_source, verify_fixture,
};

#[test]
fn native_enum_grammar_and_exhaustiveness_fail_closed() {
    fn setup(
        visibility: JavaVisibility,
    ) -> (
        portable_codegen::TargetAstBuilder<JavaDialect>,
        GeneratedTypeId,
        GeneratedValueId,
        GeneratedValueId,
        JavaTypeDeclaration,
    ) {
        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let enumeration = builder.generated_type(portable_codegen::GeneratedType {
            name: "Choice".to_owned(),
            kind: JavaDeclarationKind::Enum,
            visibility: JavaVisibility::Package,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source("enum"),
        });
        let ty = TargetTypeRef::Generated(enumeration);
        let first = builder.value(portable_codegen::GeneratedValue {
            visibility,
            name: "FIRST".to_owned(),
            ty: ty.clone(),
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source("first"),
        });
        let second = builder.value(portable_codegen::GeneratedValue {
            visibility: crate::ast::JavaVisibility::Public,
            name: "SECOND".to_owned(),
            ty,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source("second"),
        });
        let declaration = JavaTypeDeclaration {
            declared: Some(enumeration),
            kind: JavaDeclarationKind::Enum,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Choice"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![
                JavaMember::EnumConstant(JavaEnumConstant {
                    declared: first,
                    name: JavaIdentifier::from_portable("FIRST"),
                }),
                JavaMember::EnumConstant(JavaEnumConstant {
                    declared: second,
                    name: JavaIdentifier::from_portable("SECOND"),
                }),
            ],
        };
        (builder, enumeration, first, second, declaration)
    }

    let (builder, enumeration, first, second, declaration) = setup(JavaVisibility::Public);
    assert!(
        verify_fixture(
            builder,
            vec![(
                vec![
                    GeneratedSymbolId::Type(enumeration),
                    GeneratedSymbolId::Value(first),
                    GeneratedSymbolId::Value(second),
                ],
                declaration,
            )],
        )
        .is_ok()
    );

    for wrong_visibility in [false, true] {
        let visibility = if wrong_visibility {
            JavaVisibility::Private
        } else {
            JavaVisibility::Public
        };
        let (builder, enumeration, first, second, mut declaration) = setup(visibility);
        if !wrong_visibility {
            let JavaMember::EnumConstant(value) = &mut declaration.members[0] else {
                unreachable!()
            };
            value.name = JavaIdentifier::from_portable("WRONG_NAME");
        }
        let result = verify_fixture(
            builder,
            vec![(
                vec![
                    GeneratedSymbolId::Type(enumeration),
                    GeneratedSymbolId::Value(first),
                    GeneratedSymbolId::Value(second),
                ],
                declaration,
            )],
        );
        assert!(
            result.is_err(),
            "enum value name/visibility mutation was accepted"
        );
    }

    let (builder, enumeration, first, second, mut empty) = setup(JavaVisibility::Public);
    empty.members.clear();
    let diagnostics = verify_fixture(
        builder,
        vec![(
            vec![
                GeneratedSymbolId::Type(enumeration),
                GeneratedSymbolId::Value(first),
                GeneratedSymbolId::Value(second),
            ],
            empty,
        )],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidStructure
            && value.message.contains("at least one constant")
    }));

    let (builder, enumeration, first, second, declaration) = setup(JavaVisibility::Public);
    let enum_type = JavaType::Reference(JavaTypeName::Generated(enumeration));
    let selector = JavaExpr::local(enum_type.clone(), JavaIdentifier::from_portable("value"));
    let switch = JavaStmt::Switch {
        value: selector,
        arms: vec![
            JavaSwitchArm {
                pattern: JavaPattern::EnumVariant {
                    enumeration,
                    variant: first,
                },
                body: JavaBlock::new(vec![]),
            },
            JavaSwitchArm {
                pattern: JavaPattern::Default,
                body: JavaBlock::new(vec![JavaStmt::ThrowAssertion(JavaExpr::literal(
                    JavaType::known(JavaKnownType::String),
                    JavaLiteral::String("unreachable".to_owned()),
                ))]),
            },
        ],
    };
    let consumer = fixture_declaration(vec![structural_method(
        "consume",
        JavaType::primitive(JavaPrimitive::Void),
        vec![parameter(enum_type, "value")],
        JavaBlock::new(vec![switch]),
    )]);
    let diagnostics = verify_fixture(
        builder,
        vec![
            (
                vec![
                    GeneratedSymbolId::Type(enumeration),
                    GeneratedSymbolId::Value(first),
                    GeneratedSymbolId::Value(second),
                ],
                declaration,
            ),
            (vec![], consumer),
        ],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::NonExhaustiveMatch
            && value.message.contains("every declared variant")
    }));
}
