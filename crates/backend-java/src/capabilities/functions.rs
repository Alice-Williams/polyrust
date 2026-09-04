//! Java mapping for the complete `Functions` capability.

use portable_build::{CapabilityMapping, Functions};
use portable_diagnostics::Diagnostic;
use portable_ir::v0::Visibility;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{
        JavaBlock, JavaCallableRef, JavaExpr, JavaExprKind, JavaMethod, JavaMethodDeclaration,
        JavaModifier, JavaParameter, JavaPrecedence, JavaType,
    },
    dialect::JavaDialect,
    lower::{identifier, visibility_modifier},
};

#[doc(hidden)]
pub struct JavaFunctionDeclarationInput {
    pub(crate) declared: JavaMethodDeclaration,
    pub(crate) visibility: Visibility,
    pub(crate) name: String,
    pub(crate) parameters: Vec<JavaParameter>,
    pub(crate) return_type: JavaType,
    pub(crate) body: JavaBlock,
}

#[doc(hidden)]
pub enum JavaFunctionsInput {
    Declaration(Box<JavaFunctionDeclarationInput>),
    Local {
        ty: JavaType,
        name: String,
    },
    Call {
        result: JavaType,
        callable: Box<JavaCallableRef>,
        arguments: Vec<JavaExpr>,
    },
}

#[doc(hidden)]
pub enum JavaFunctionsNode {
    Declaration(JavaMethod),
    Expression(JavaExpr),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaFunctions;

impl sealed::JavaCapabilityMapping for JavaFunctions {}
impl JavaCapabilityMapping for JavaFunctions {}

impl CapabilityMapping<JavaDialect> for JavaFunctions {
    type Capability = Functions;
    type Context = ();
    type Input = JavaFunctionsInput;
    type Output = JavaFunctionsNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaFunctionsInput::Declaration(input) => JavaFunctionsNode::Declaration(JavaMethod {
                declared: input.declared,
                annotations: vec![],
                modifiers: vec![visibility_modifier(input.visibility), JavaModifier::Static],
                type_parameters: vec![],
                return_type: input.return_type,
                name: identifier(&input.name),
                parameters: input.parameters,
                body: Some(input.body),
            }),
            JavaFunctionsInput::Local { ty, name } => {
                JavaFunctionsNode::Expression(JavaExpr::local(ty, identifier(&name)))
            }
            JavaFunctionsInput::Call {
                result,
                callable,
                arguments,
            } => JavaFunctionsNode::Expression(JavaExpr {
                ty: result,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Call {
                    callable: *callable,
                    receiver: None,
                    arguments,
                },
            }),
        })
    }
}
