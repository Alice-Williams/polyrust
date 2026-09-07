use super::{
    DiagnosticCode, GeneratedSymbolId, JavaBlock, JavaConstructor, JavaDeclarationKind,
    JavaDialect, JavaExpr, JavaExprKind, JavaField, JavaFieldRef, JavaHeritage, JavaIdentifier,
    JavaKnownType, JavaLiteral, JavaMember, JavaMethod, JavaMethodDeclaration, JavaModifier,
    JavaPrecedence, JavaPrimitive, JavaRecordComponent, JavaRecordComponentOrigin,
    JavaRuntimeMember, JavaStmt, JavaType, JavaTypeDeclaration, JavaTypeName, JavaValueRef,
    JavaVisibility, fixture_core_field, fixture_declaration, parameter, verifier_source,
    verify_fixture,
};

#[test]
fn this_references_must_match_the_lexical_owner() {
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let owner = builder.generated_type(portable_codegen::GeneratedType {
        name: "Owner".to_owned(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Package,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::TestHarness,
        ),
        source: verifier_source("owner"),
    });
    let mut declaration = fixture_declaration(vec![JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public],
        type_parameters: vec![],
        return_type: JavaType::known(JavaKnownType::String),
        name: JavaIdentifier::from_portable("selfValue"),
        parameters: vec![],
        body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: JavaType::known(JavaKnownType::String),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::This),
        }))])),
    })]);
    declaration.declared = Some(owner);
    declaration.name = JavaIdentifier::from_portable("Owner");
    let diagnostics = verify_fixture(
        builder,
        vec![(vec![GeneratedSymbolId::Type(owner)], declaration)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::UnresolvedReference
            && value.message.contains("declaring Java owner")
    }));

    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let owner = builder.generated_type(portable_codegen::GeneratedType {
        name: "Owner".to_owned(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Package,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::TestHarness,
        ),
        source: verifier_source("owner-positive"),
    });
    let owner_type = JavaType::Reference(JavaTypeName::Generated(owner));
    let mut declaration = fixture_declaration(vec![JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public],
        type_parameters: vec![],
        return_type: owner_type.clone(),
        name: JavaIdentifier::from_portable("selfValue"),
        parameters: vec![],
        body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: owner_type,
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::This),
        }))])),
    })]);
    declaration.declared = Some(owner);
    declaration.name = JavaIdentifier::from_portable("Owner");
    assert!(
        verify_fixture(
            builder,
            vec![(vec![GeneratedSymbolId::Type(owner)], declaration,)],
        )
        .is_ok()
    );
}

#[test]
fn structural_fields_require_declared_type_and_final_assignment_context() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let owner = JavaType::known(JavaKnownType::RuntimeError);
    let string = JavaType::known(JavaKnownType::String);
    let this = || JavaExpr {
        ty: owner.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::This),
    };
    let field = |name: &str, ty: JavaType| JavaExpr {
        ty: ty.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Field {
            receiver: Box::new(this()),
            field: JavaFieldRef::Structural {
                name: JavaIdentifier::from_portable(name),
                ty,
            },
        },
    };
    let valid = JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("PolyError"),
        type_parameters: vec![],
        record_components: vec![JavaRecordComponent {
            origin: JavaRecordComponentOrigin::Runtime(JavaRuntimeMember::ErrorCode),
            ty: string.clone(),
            name: JavaIdentifier::from_portable("code"),
        }],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Constructor(JavaConstructor {
            modifiers: vec![],
            name: JavaIdentifier::from_portable("PolyError"),
            parameters: vec![parameter(string.clone(), "code")],
            body: JavaBlock::new(vec![JavaStmt::Assign {
                target: field("code", string.clone()),
                value: JavaExpr::local(string.clone(), JavaIdentifier::from_portable("code")),
            }]),
        })],
    };
    assert!(
        verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], valid)],
        )
        .is_ok()
    );

    let mut invalid = fixture_declaration(vec![
        JavaMember::Field(JavaField {
            declared: None,
            modifiers: vec![JavaModifier::Private, JavaModifier::Final],
            ty: int.clone(),
            name: JavaIdentifier::from_portable("count"),
            initializer: None,
        }),
        JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![],
            modifiers: vec![JavaModifier::Public],
            type_parameters: vec![],
            return_type: JavaType::primitive(JavaPrimitive::Void),
            name: JavaIdentifier::from_portable("mutate"),
            parameters: vec![],
            body: Some(JavaBlock::new(vec![JavaStmt::Assign {
                target: JavaExpr {
                    ty: int.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Field {
                        receiver: Box::new(JavaExpr {
                            ty: JavaType::known(JavaKnownType::String),
                            precedence: JavaPrecedence::Primary,
                            kind: JavaExprKind::Value(JavaValueRef::This),
                        }),
                        field: JavaFieldRef::Structural {
                            name: JavaIdentifier::from_portable("missing"),
                            ty: int.clone(),
                        },
                    },
                },
                value: JavaExpr::literal(int, JavaLiteral::I32(1)),
            }])),
        }),
    ]);
    invalid.name = JavaIdentifier::from_portable("Fixture");
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], invalid)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::UnresolvedReference
            && value.message.contains("structural Java field")
    }));
}

#[test]
fn generated_record_fields_are_final_outside_their_canonical_constructor() {
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let record = builder.generated_type(portable_codegen::GeneratedType {
        name: "Value".to_owned(),
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Package,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::TestHarness,
        ),
        source: verifier_source("field-record"),
    });
    let field_id = fixture_core_field();
    let int = JavaType::primitive(JavaPrimitive::Int);
    let owner = JavaType::Reference(JavaTypeName::Generated(record));
    let generated_field = || JavaExpr {
        ty: int.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Field {
            receiver: Box::new(JavaExpr {
                ty: owner.clone(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::This),
            }),
            field: JavaFieldRef::Generated {
                owner: record,
                field: field_id,
                name: JavaIdentifier::from_portable("value"),
                ty: int.clone(),
            },
        },
    };
    let declaration = JavaTypeDeclaration {
        declared: Some(record),
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("Value"),
        type_parameters: vec![],
        record_components: vec![JavaRecordComponent {
            origin: JavaRecordComponentOrigin::Core(field_id),
            ty: int.clone(),
            name: JavaIdentifier::from_portable("value"),
        }],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![
            JavaMember::Constructor(JavaConstructor {
                modifiers: vec![],
                name: JavaIdentifier::from_portable("Value"),
                parameters: vec![parameter(int.clone(), "value")],
                body: JavaBlock::new(vec![JavaStmt::Assign {
                    target: generated_field(),
                    value: JavaExpr::local(int.clone(), JavaIdentifier::from_portable("value")),
                }]),
            }),
            JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Structural,
                annotations: vec![],
                modifiers: vec![JavaModifier::Public],
                type_parameters: vec![],
                return_type: JavaType::primitive(JavaPrimitive::Void),
                name: JavaIdentifier::from_portable("mutate"),
                parameters: vec![],
                body: Some(JavaBlock::new(vec![JavaStmt::Assign {
                    target: generated_field(),
                    value: JavaExpr::literal(int, JavaLiteral::I32(2)),
                }])),
            }),
            JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Structural,
                annotations: vec![],
                modifiers: vec![JavaModifier::Public],
                type_parameters: vec![],
                return_type: JavaType::primitive(JavaPrimitive::Int),
                name: JavaIdentifier::from_portable("missing"),
                parameters: vec![],
                body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                    ty: JavaType::primitive(JavaPrimitive::Int),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Field {
                        receiver: Box::new(JavaExpr {
                            ty: owner,
                            precedence: JavaPrecedence::Primary,
                            kind: JavaExprKind::Value(JavaValueRef::This),
                        }),
                        field: JavaFieldRef::Generated {
                            owner: record,
                            field: field_id,
                            name: JavaIdentifier::from_portable("missing"),
                            ty: JavaType::primitive(JavaPrimitive::Int),
                        },
                    },
                }))])),
            }),
        ],
    };
    let diagnostics = verify_fixture(
        builder,
        vec![(vec![GeneratedSymbolId::Type(record)], declaration)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("declaring constructor")
    }));
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::UnresolvedReference
            && value.message.contains("generated Java field reference")
    }));
}
