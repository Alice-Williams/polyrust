//! Java mapping for the complete `LocalBindings` capability.

mod mapping_plan;

use portable_build::{CapabilityMapping, LocalBindings};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaExpr, JavaLocalFinality, JavaStmt, JavaType},
    dialect::JavaDialect,
    lower::identifier,
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaLocalBindingsInput {
    Bind {
        name: String,
        ty: JavaType,
        value: Box<JavaExpr>,
    },
    Read {
        name: String,
        ty: JavaType,
    },
}

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaLocalBindingsNode {
    Statement(Box<JavaStmt>),
    Expression(Box<JavaExpr>),
}

impl sealed::JavaMappingOutput for JavaLocalBindingsNode {}
impl super::support::JavaMappingOutput for JavaLocalBindingsNode {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaLocalBindings;

impl sealed::JavaCapabilityMapping for JavaLocalBindings {}
impl JavaCapabilityMapping for JavaLocalBindings {
    type Plan = mapping_plan::Plan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        mapping_plan::select(input)
    }
}

impl CapabilityMapping<JavaDialect> for JavaLocalBindings {
    type Capability = LocalBindings;
    type Context = ();
    type Input = JavaLocalBindingsInput;
    type Output = JavaLocalBindingsNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaLocalBindingsInput::Bind { name, ty, value } => {
                JavaLocalBindingsNode::Statement(Box::new(JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty,
                    name: identifier(&name),
                    value: Some(*value),
                }))
            }
            JavaLocalBindingsInput::Read { name, ty } => {
                JavaLocalBindingsNode::Expression(Box::new(JavaExpr::local(ty, identifier(&name))))
            }
        })
    }
}
