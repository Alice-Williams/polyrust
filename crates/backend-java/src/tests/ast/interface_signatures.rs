use super::{
    DiagnosticCode, GeneratedSymbolId, JavaAnnotation, JavaBlock, JavaDeclarationKind, JavaDialect,
    JavaExpr, JavaHeritage, JavaIdentifier, JavaInvocationKind, JavaKnownType, JavaLiteral,
    JavaMember, JavaMethod, JavaMethodDeclaration, JavaModifier, JavaPrimitive, JavaStmt, JavaType,
    JavaTypeDeclaration, JavaTypeName, JavaVisibility, TargetTypeRef,
    fixture_core_implementation_method, parameter, verifier_source, verify_fixture,
};

#[test]
fn generated_interface_implementations_require_exact_generic_signatures() {
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let interface = builder.generated_type(portable_codegen::GeneratedType {
        name: "ReviewInterface".to_owned(),
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Package,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::InterfaceAdapter,
        ),
        source: verifier_source("generic-interface"),
    });
    let implementation = builder.generated_type(portable_codegen::GeneratedType {
        name: "ForgedImplementation".to_owned(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Package,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::InterfaceAdapter,
        ),
        source: verifier_source("generic-implementation"),
    });
    let strings = JavaType::generic(
        JavaKnownType::List,
        vec![JavaType::known(JavaKnownType::String)],
    );
    let integers = JavaType::generic(
        JavaKnownType::List,
        vec![JavaType::known(JavaKnownType::Integer)],
    );
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let method = builder.interface_method(portable_codegen::GeneratedInterfaceMethod {
        owner: interface,
        name: "render".to_owned(),
        signature: portable_codegen::TargetCallableSignature {
            invocation: JavaInvocationKind::Instance,
            receiver: Some(TargetTypeRef::Generated(interface)),
            parameters: vec![JavaDialect.registered_type(&strings)],
            return_type: JavaDialect.registered_type(&boolean),
        },
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::InterfaceAdapter,
        ),
        source: verifier_source("generic-method"),
    });
    let interface_declaration = JavaTypeDeclaration {
        declared: Some(interface),
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("ReviewInterface"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Interface(method),
            annotations: vec![],
            modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
            type_parameters: vec![],
            return_type: boolean.clone(),
            name: JavaIdentifier::from_portable("render"),
            parameters: vec![parameter(strings, "values")],
            body: None,
        })],
    };
    let (core_method, witness) = fixture_core_implementation_method(implementation, method);
    let implementation_declaration = JavaTypeDeclaration {
        declared: Some(implementation),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("ForgedImplementation"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::Interfaces(vec![JavaType::Reference(JavaTypeName::Generated(
            interface,
        ))]),
        permits: vec![],
        members: vec![JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Implementation {
                method: core_method,
                interface: method,
                witness,
            },
            annotations: vec![JavaAnnotation::Override],
            modifiers: vec![JavaModifier::Public],
            type_parameters: vec![],
            return_type: boolean,
            name: JavaIdentifier::from_portable("render"),
            parameters: vec![parameter(integers, "values")],
            body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
                JavaExpr::literal(
                    JavaType::primitive(JavaPrimitive::Boolean),
                    JavaLiteral::Boolean(true),
                ),
            ))])),
        })],
    };
    let diagnostics = verify_fixture(
        builder,
        vec![
            (
                vec![
                    GeneratedSymbolId::Type(interface),
                    GeneratedSymbolId::InterfaceMethod(method),
                ],
                interface_declaration,
            ),
            (
                vec![GeneratedSymbolId::Type(implementation)],
                implementation_declaration,
            ),
        ],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidInvocation
            && diagnostic
                .message
                .contains("authoritative registered callable")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InvalidStructure
            && diagnostic.message.contains("no verified instance override")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::InterfaceNonconformance
            && diagnostic.message.contains("exactly once")
    }));
}
