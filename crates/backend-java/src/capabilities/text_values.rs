//! Java mapping for `TextValues`.

mod lowering;
mod mapping_plan;
pub(crate) use lowering::text_value;

use portable_build::{CapabilityMapping, TextValues};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, JavaValueNode, sealed};
use crate::{
    ast::{JavaKnownType, JavaType},
    dialect::JavaDialect,
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaTextValuesInput {
    Type,
    Value(String),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaTextValues;

impl sealed::JavaCapabilityMapping for JavaTextValues {}
impl JavaCapabilityMapping for JavaTextValues {
    type Plan = mapping_plan::Plan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        mapping_plan::select(input)
    }
}

impl CapabilityMapping<JavaDialect> for JavaTextValues {
    type Capability = TextValues;
    type Context = ();
    type Input = JavaTextValuesInput;
    type Output = JavaValueNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaTextValuesInput::Type => {
                JavaValueNode::Type(JavaType::known(JavaKnownType::String))
            }
            JavaTextValuesInput::Value(value) => {
                JavaValueNode::Expression(Box::new(lowering::text_value(&value)))
            }
        })
    }
}
