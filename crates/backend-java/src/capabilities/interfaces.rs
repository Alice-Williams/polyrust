//! Java mapping for the complete `Interfaces` capability.

use portable_build::{CapabilityMapping, Interfaces};
use portable_codegen::{GeneratedInterfaceMethodId, GeneratedTypeId};
use portable_core_ir::CoreImplementationMethodId;
use portable_diagnostics::Diagnostic;
use portable_ir::v0::Visibility;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{
        JavaAnnotation, JavaBlock, JavaCallableRef, JavaDeclarationKind, JavaExpr, JavaExprKind,
        JavaHeritage, JavaMember, JavaMemberOrigin, JavaMethod, JavaMethodDeclaration,
        JavaMethodSignature, JavaModifier, JavaParameter, JavaPrecedence, JavaType,
        JavaTypeDeclaration,
    },
    dialect::JavaDialect,
    lower::{identifier, java_visibility, member_call},
};

#[doc(hidden)]
pub struct JavaInterfaceMethodInput {
    pub(crate) declared: GeneratedInterfaceMethodId,
    pub(crate) name: String,
    pub(crate) parameters: Vec<JavaParameter>,
    pub(crate) return_type: JavaType,
}

#[doc(hidden)]
pub struct JavaInterfaceDeclarationInput {
    pub(crate) declared: GeneratedTypeId,
    pub(crate) visibility: Visibility,
    pub(crate) name: String,
    pub(crate) permits: Vec<JavaType>,
    pub(crate) methods: Vec<JavaInterfaceMethodInput>,
}

#[doc(hidden)]
pub struct JavaInterfaceImplementationInput {
    pub(crate) method: CoreImplementationMethodId,
    pub(crate) interface_method: GeneratedInterfaceMethodId,
    pub(crate) name: String,
    pub(crate) parameters: Vec<JavaParameter>,
    pub(crate) return_type: JavaType,
    pub(crate) body: JavaBlock,
}

#[doc(hidden)]
pub struct JavaInterfaceCallInput {
    pub(crate) receiver: JavaExpr,
    pub(crate) arguments: Vec<JavaExpr>,
    pub(crate) result: JavaType,
    pub(crate) symbol: GeneratedInterfaceMethodId,
    pub(crate) signature: JavaMethodSignature,
}

#[doc(hidden)]
pub struct JavaConcreteInterfaceCallInput {
    pub(crate) receiver: JavaExpr,
    pub(crate) name: String,
    pub(crate) arguments: Vec<JavaExpr>,
    pub(crate) result: JavaType,
    pub(crate) method: CoreImplementationMethodId,
}

#[doc(hidden)]
pub enum JavaInterfacesInput {
    Declaration(Box<JavaInterfaceDeclarationInput>),
    Implementation(Box<JavaInterfaceImplementationInput>),
    Coerce {
        value: Box<JavaExpr>,
        result: JavaType,
    },
    ConcreteCall(Box<JavaConcreteInterfaceCallInput>),
    InterfaceCall(Box<JavaInterfaceCallInput>),
}

#[doc(hidden)]
pub enum JavaInterfacesNode {
    Declaration(Box<JavaTypeDeclaration>),
    Method(Box<JavaMethod>),
    Expression(Box<JavaExpr>),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaInterfaces;

impl sealed::JavaCapabilityMapping for JavaInterfaces {}
impl JavaCapabilityMapping for JavaInterfaces {}

impl CapabilityMapping<JavaDialect> for JavaInterfaces {
    type Capability = Interfaces;
    type Context = ();
    type Input = JavaInterfacesInput;
    type Output = JavaInterfacesNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaInterfacesInput::Declaration(input) => {
                let members = input
                    .methods
                    .into_iter()
                    .map(|method| {
                        JavaMember::Method(JavaMethod {
                            declared: JavaMethodDeclaration::Interface(method.declared),
                            annotations: vec![],
                            modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
                            type_parameters: vec![],
                            return_type: method.return_type,
                            name: identifier(&method.name),
                            parameters: method.parameters,
                            body: None,
                        })
                    })
                    .collect();
                JavaInterfacesNode::Declaration(Box::new(JavaTypeDeclaration {
                    declared: Some(input.declared),
                    kind: JavaDeclarationKind::SealedInterface,
                    visibility: java_visibility(input.visibility),
                    modifiers: vec![JavaModifier::Static],
                    name: identifier(&input.name),
                    type_parameters: vec![],
                    record_components: vec![],
                    heritage: JavaHeritage::None,
                    permits: input.permits,
                    members,
                }))
            }
            JavaInterfacesInput::Implementation(input) => {
                JavaInterfacesNode::Method(Box::new(JavaMethod {
                    declared: JavaMethodDeclaration::Implementation {
                        method: input.method,
                        interface: input.interface_method,
                    },
                    annotations: vec![JavaAnnotation::Override],
                    modifiers: vec![JavaModifier::Public],
                    type_parameters: vec![],
                    return_type: input.return_type,
                    name: identifier(&input.name),
                    parameters: input.parameters,
                    body: Some(input.body),
                }))
            }
            JavaInterfacesInput::Coerce { value, result } => {
                JavaInterfacesNode::Expression(Box::new(JavaExpr {
                    ty: result.clone(),
                    precedence: JavaPrecedence::Unary,
                    kind: JavaExprKind::Cast {
                        target: result,
                        value,
                    },
                }))
            }
            JavaInterfacesInput::ConcreteCall(input) => {
                JavaInterfacesNode::Expression(Box::new(member_call(
                    input.receiver,
                    &input.name,
                    input.arguments,
                    input.result,
                    JavaMemberOrigin::GeneratedImplementation(input.method),
                )))
            }
            JavaInterfacesInput::InterfaceCall(input) => {
                JavaInterfacesNode::Expression(Box::new(JavaExpr {
                    ty: input.result,
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Call {
                        callable: JavaCallableRef::Interface {
                            symbol: input.symbol,
                            signature: input.signature,
                        },
                        receiver: Some(Box::new(input.receiver)),
                        arguments: input.arguments,
                    },
                }))
            }
        })
    }
}
