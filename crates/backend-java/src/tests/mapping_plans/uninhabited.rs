//! Target-only empty-interface bodies contain no opaque input holes.
use super::*;
use crate::{
    ast::*,
    capabilities as c,
    dialect::{JavaInvocationKind, JavaKnownConstructor},
    lower::string_literal,
};
use portable_codegen::{
    GeneratedInterfaceMethod, GeneratedOrigin, GeneratedType, SynthesisReason, TargetAstBuilder,
    TargetCallableSignature, TargetTypeRef,
};
use portable_diagnostics::SourceRef;
use portable_ir::v0::Visibility;

#[derive(Clone, Copy)]
enum Fault {
    ResultType,
    Owner,
    Signature,
    ArgumentType,
    ArgumentNode,
    Constructor,
}

#[test]
fn uninhabited_method_construction_is_fully_owned() {
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let source = SourceRef::logical(["uninhabited-plan"]);
    let interface = builder.generated_type(GeneratedType {
        name: "Empty".into(),
        kind: JavaDeclarationKind::SealedInterface,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
        source: source.clone(),
    });
    let synthetic = builder.generated_type(GeneratedType {
        name: "UninhabitedEmpty".into(),
        kind: JavaDeclarationKind::UninhabitedEnum(interface),
        visibility: JavaVisibility::Private,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::UninhabitedInterface),
        source: source.clone(),
    });
    let method = builder.interface_method(GeneratedInterfaceMethod {
        owner: interface,
        name: "read".into(),
        signature: TargetCallableSignature {
            invocation: JavaInvocationKind::Instance,
            receiver: Some(TargetTypeRef::Generated(interface)),
            parameters: vec![],
            return_type: TargetTypeRef::Known(JavaKnownType::String),
        },
        origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
        source,
    });
    for fault in [
        Fault::ResultType,
        Fault::Owner,
        Fault::Signature,
        Fault::ArgumentType,
        Fault::ArgumentNode,
        Fault::Constructor,
    ] {
        let input = c::interfaces::JavaInterfacesInput::Declaration(Box::new(
            c::interfaces::JavaInterfaceDeclarationInput {
                declared: interface,
                visibility: Visibility::Public,
                name: "Empty".into(),
                permits: vec![JavaType::Reference(JavaTypeName::Generated(synthetic))],
                methods: vec![c::interfaces::JavaInterfaceMethodInput {
                    declared: method,
                    name: "read".into(),
                    parameters: vec![],
                    return_type: JavaType::known(JavaKnownType::String),
                }],
                uninhabited: Some(c::interfaces::JavaUninhabitedInterfaceInput {
                    declared: synthetic,
                    name: "UninhabitedEmpty".into(),
                }),
            },
        ));
        let (plan, mut output) = checked(c::JavaInterfaces, input);
        let c::interfaces::JavaInterfacesNode::Declaration(declarations) = &mut output else {
            panic!()
        };
        let JavaMember::Method(method) = &mut declarations[1].members[0] else {
            panic!()
        };
        let JavaStmt::Throw(value) = &mut method.body.as_mut().unwrap().statements[0] else {
            panic!()
        };
        let JavaExprKind::New {
            constructor:
                JavaConstructorRef::Known {
                    constructor,
                    owner,
                    parameters,
                },
            arguments,
        } = &mut value.kind
        else {
            panic!()
        };
        match fault {
            Fault::ResultType => value.ty = JavaType::known(JavaKnownType::String),
            Fault::Owner => *owner = JavaType::known(JavaKnownType::String),
            Fault::Signature => parameters.clear(),
            Fault::ArgumentType => arguments[0].ty = JavaType::primitive(JavaPrimitive::Int),
            Fault::ArgumentNode => arguments[0] = string_literal("different body"),
            Fault::Constructor => *constructor = JavaKnownConstructor::RuntimeUnit,
        }
        assert!(!plan.verify_output(&output));
    }
}
