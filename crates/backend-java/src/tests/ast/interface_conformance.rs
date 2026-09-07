use super::{
    DiagnosticCode, GeneratedSymbolId, JavaDeclarationKind, JavaDialect, JavaHeritage,
    JavaIdentifier, JavaInvocationKind, JavaMember, JavaMethod, JavaMethodDeclaration,
    JavaModifier, JavaPrimitive, JavaType, JavaTypeDeclaration, JavaTypeName, JavaVisibility,
    TargetTypeRef, verifier_source, verify_fixture,
};

#[test]
fn generated_interface_conformance_requires_every_declared_method() {
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let interface = builder.generated_type(portable_codegen::GeneratedType {
        name: "Service".to_owned(),
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Package,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::InterfaceAdapter,
        ),
        source: verifier_source("interface"),
    });
    let record = builder.generated_type(portable_codegen::GeneratedType {
        name: "Implementation".to_owned(),
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Package,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::InterfaceAdapter,
        ),
        source: verifier_source("record"),
    });
    let method = builder.interface_method(portable_codegen::GeneratedInterfaceMethod {
        owner: interface,
        name: "value".to_owned(),
        signature: portable_codegen::TargetCallableSignature {
            invocation: JavaInvocationKind::Instance,
            receiver: Some(TargetTypeRef::Generated(interface)),
            parameters: vec![],
            return_type: TargetTypeRef::Primitive(JavaPrimitive::Int),
        },
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::InterfaceAdapter,
        ),
        source: verifier_source("method"),
    });
    let interface_declaration = JavaTypeDeclaration {
        declared: Some(interface),
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("Service"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Interface(method),
            annotations: vec![],
            modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
            type_parameters: vec![],
            return_type: JavaType::primitive(JavaPrimitive::Int),
            name: JavaIdentifier::from_portable("value"),
            parameters: vec![],
            body: None,
        })],
    };
    let record_declaration = JavaTypeDeclaration {
        declared: Some(record),
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("Implementation"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::Interfaces(vec![JavaType::Reference(JavaTypeName::Generated(
            interface,
        ))]),
        permits: vec![],
        members: vec![],
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
            (vec![GeneratedSymbolId::Type(record)], record_declaration),
        ],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InterfaceNonconformance
            && value.message.contains("exactly once")
    }));
}

#[test]
fn generated_interfaces_reject_unregistered_structural_methods() {
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let interface = builder.generated_type(portable_codegen::GeneratedType {
        name: "StructuralEscape".to_owned(),
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Package,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::InterfaceAdapter,
        ),
        source: verifier_source("structural-interface"),
    });
    let declaration = JavaTypeDeclaration {
        declared: Some(interface),
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("StructuralEscape"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![],
            modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
            type_parameters: vec![],
            return_type: JavaType::primitive(JavaPrimitive::Int),
            name: JavaIdentifier::from_portable("hidden"),
            parameters: vec![],
            body: None,
        })],
    };
    let diagnostics = verify_fixture(
        builder,
        vec![(vec![GeneratedSymbolId::Type(interface)], declaration)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidStructure
            && value.message.contains("registered interface method")
    }));
}

#[test]
fn registered_clone_methods_cannot_bypass_object_collision_checks() {
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let interface = builder.generated_type(portable_codegen::GeneratedType {
        name: "CloneEscape".to_owned(),
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Package,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::InterfaceAdapter,
        ),
        source: verifier_source("clone-interface"),
    });
    let method = builder.interface_method(portable_codegen::GeneratedInterfaceMethod {
        owner: interface,
        name: "clone".to_owned(),
        signature: portable_codegen::TargetCallableSignature {
            invocation: JavaInvocationKind::Instance,
            receiver: Some(TargetTypeRef::Generated(interface)),
            parameters: vec![],
            return_type: TargetTypeRef::Primitive(JavaPrimitive::Int),
        },
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::InterfaceAdapter,
        ),
        source: verifier_source("clone-method"),
    });
    let declaration = JavaTypeDeclaration {
        declared: Some(interface),
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("CloneEscape"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Interface(method),
            annotations: vec![],
            modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
            type_parameters: vec![],
            return_type: JavaType::primitive(JavaPrimitive::Int),
            name: JavaIdentifier::from_portable("clone"),
            parameters: vec![],
            body: None,
        })],
    };
    let diagnostics = verify_fixture(
        builder,
        vec![(
            vec![
                GeneratedSymbolId::Type(interface),
                GeneratedSymbolId::InterfaceMethod(method),
            ],
            declaration,
        )],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidStructure
            && value.message.contains("inherited Object method")
    }));
}
