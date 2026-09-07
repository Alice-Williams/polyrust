use super::{
    DiagnosticCode, GeneratedSymbolId, JavaDeclarationKind, JavaDialect, JavaHeritage,
    JavaIdentifier, JavaType, JavaTypeDeclaration, JavaTypeName, JavaVisibility, verifier_source,
    verify_fixture,
};

#[test]
fn sealed_permits_are_unique_and_exactly_match_implementors() {
    let setup = |valid: bool| {
        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let interface = builder.generated_type(portable_codegen::GeneratedType {
            name: "Shape".to_owned(),
            kind: JavaDeclarationKind::SealedInterface,
            visibility: JavaVisibility::Package,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source("sealed-interface"),
        });
        let implementation = builder.generated_type(portable_codegen::GeneratedType {
            name: "Circle".to_owned(),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source("sealed-implementation"),
        });
        let unrelated = builder.generated_type(portable_codegen::GeneratedType {
            name: "Square".to_owned(),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            origin: portable_codegen::GeneratedOrigin::Synthesized(
                portable_codegen::SynthesisReason::TestHarness,
            ),
            source: verifier_source("sealed-unrelated"),
        });
        let interface_type = JavaType::Reference(JavaTypeName::Generated(interface));
        let implementation_type = JavaType::Reference(JavaTypeName::Generated(implementation));
        let unrelated_type = JavaType::Reference(JavaTypeName::Generated(unrelated));
        let interface_declaration = JavaTypeDeclaration {
            declared: Some(interface),
            kind: JavaDeclarationKind::SealedInterface,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Shape"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: if valid {
                vec![implementation_type]
            } else {
                vec![unrelated_type.clone(), unrelated_type]
            },
            members: vec![],
        };
        let implementation_declaration = JavaTypeDeclaration {
            declared: Some(implementation),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Circle"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::Interfaces(vec![interface_type]),
            permits: vec![],
            members: vec![],
        };
        let unrelated_declaration = JavaTypeDeclaration {
            declared: Some(unrelated),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("Square"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![],
        };
        (
            builder,
            vec![
                (
                    vec![GeneratedSymbolId::Type(interface)],
                    interface_declaration,
                ),
                (
                    vec![GeneratedSymbolId::Type(implementation)],
                    implementation_declaration,
                ),
                (
                    vec![GeneratedSymbolId::Type(unrelated)],
                    unrelated_declaration,
                ),
            ],
        )
    };
    let (builder, declarations) = setup(false);
    let diagnostics = verify_fixture(builder, declarations).unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::DuplicateDeclaration
            && value.message.contains("permits entry is repeated")
    }));
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InterfaceNonconformance
            && value.message.contains("exactly name every implementing")
    }));

    let (builder, declarations) = setup(true);
    assert!(verify_fixture(builder, declarations).is_ok());
}
