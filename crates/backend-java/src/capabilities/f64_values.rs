//! Java mapping for `F64Values`.

mod mapping_plan;

use portable_build::{CapabilityMapping, F64Values};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, JavaValueNode, sealed};
use crate::{
    ast::{JavaPrimitive, JavaType},
    dialect::JavaDialect,
    lower::f64_literal,
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaF64ValuesInput {
    Type,
    Value(u64),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaF64Values;

impl sealed::JavaCapabilityMapping for JavaF64Values {}
impl JavaCapabilityMapping for JavaF64Values {
    type Plan = mapping_plan::Plan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        mapping_plan::select(input)
    }
}

impl CapabilityMapping<JavaDialect> for JavaF64Values {
    type Capability = F64Values;
    type Context = ();
    type Input = JavaF64ValuesInput;
    type Output = JavaValueNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaF64ValuesInput::Type => {
                JavaValueNode::Type(JavaType::primitive(JavaPrimitive::Double))
            }
            JavaF64ValuesInput::Value(value) => {
                JavaValueNode::Expression(Box::new(f64_literal(value)))
            }
        })
    }
}
