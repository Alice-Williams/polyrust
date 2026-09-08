//! Java mapping for the complete `Constants` capability.

mod mapping_plan;

use portable_build::{CapabilityMapping, Constants};
use portable_codegen::{GeneratedSymbolId, GeneratedValueId};
use portable_diagnostics::Diagnostic;
use portable_ir::v0::Visibility;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{
        JavaExpr, JavaExprKind, JavaField, JavaModifier, JavaPrecedence, JavaType, JavaValueRef,
    },
    dialect::JavaDialect,
    lower::{identifier, visibility_modifier},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaConstantsInput {
    Declaration {
        declared: GeneratedValueId,
        visibility: Visibility,
        name: String,
        ty: JavaType,
        initializer: Box<JavaExpr>,
    },
    Reference {
        symbol: GeneratedValueId,
        result: JavaType,
    },
}

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaConstantsNode {
    Declaration(JavaField),
    Expression(JavaExpr),
}

impl sealed::JavaMappingOutput for JavaConstantsNode {}
impl super::support::JavaMappingOutput for JavaConstantsNode {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaConstants;

impl sealed::JavaCapabilityMapping for JavaConstants {}
impl JavaCapabilityMapping for JavaConstants {
    type Plan = mapping_plan::Plan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        mapping_plan::select(input)
    }
}

impl CapabilityMapping<JavaDialect> for JavaConstants {
    type Capability = Constants;
    type Context = ();
    type Input = JavaConstantsInput;
    type Output = JavaConstantsNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaConstantsInput::Declaration {
                declared,
                visibility,
                name,
                ty,
                initializer,
            } => JavaConstantsNode::Declaration(JavaField {
                declared: Some(declared),
                modifiers: vec![
                    visibility_modifier(visibility),
                    JavaModifier::Static,
                    JavaModifier::Final,
                ],
                ty,
                name: identifier(&name),
                initializer: Some(*initializer),
            }),
            JavaConstantsInput::Reference { symbol, result } => {
                JavaConstantsNode::Expression(JavaExpr {
                    ty: result,
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(
                        symbol,
                    ))),
                })
            }
        })
    }
}
