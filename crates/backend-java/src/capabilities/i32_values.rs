//! Java mapping for `I32Values`.

mod mapping_plan;

use portable_build::{CapabilityMapping, I32Values};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, JavaValueNode, sealed};
use crate::{
    ast::{JavaPrimitive, JavaType},
    dialect::JavaDialect,
    lower::i32_literal,
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaI32ValuesInput {
    Type,
    Value(i32),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaI32Values;

impl sealed::JavaCapabilityMapping for JavaI32Values {}
impl JavaCapabilityMapping for JavaI32Values {
    type Plan = mapping_plan::Plan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        mapping_plan::select(input)
    }
}

impl CapabilityMapping<JavaDialect> for JavaI32Values {
    type Capability = I32Values;
    type Context = ();
    type Input = JavaI32ValuesInput;
    type Output = JavaValueNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaI32ValuesInput::Type => {
                JavaValueNode::Type(JavaType::primitive(JavaPrimitive::Int))
            }
            JavaI32ValuesInput::Value(value) => {
                JavaValueNode::Expression(Box::new(i32_literal(value)))
            }
        })
    }
}
