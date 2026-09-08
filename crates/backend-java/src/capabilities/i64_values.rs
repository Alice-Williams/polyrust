//! Java mapping for `I64Values`.

mod mapping_plan;

use portable_build::{CapabilityMapping, I64Values};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, JavaValueNode, sealed};
use crate::{
    ast::{JavaPrimitive, JavaType},
    dialect::JavaDialect,
    lower::i64_literal,
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaI64ValuesInput {
    Type,
    Value(i64),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaI64Values;

impl sealed::JavaCapabilityMapping for JavaI64Values {}
impl JavaCapabilityMapping for JavaI64Values {
    type Plan = mapping_plan::Plan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        mapping_plan::select(input)
    }
}

impl CapabilityMapping<JavaDialect> for JavaI64Values {
    type Capability = I64Values;
    type Context = ();
    type Input = JavaI64ValuesInput;
    type Output = JavaValueNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaI64ValuesInput::Type => {
                JavaValueNode::Type(JavaType::primitive(JavaPrimitive::Long))
            }
            JavaI64ValuesInput::Value(value) => {
                JavaValueNode::Expression(Box::new(i64_literal(value)))
            }
        })
    }
}
