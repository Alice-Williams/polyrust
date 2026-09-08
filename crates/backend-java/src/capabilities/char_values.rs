//! Java mapping for `CharValues`.

mod mapping_plan;

use portable_build::{CapabilityMapping, CharValues};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, JavaValueNode, sealed};
use crate::{
    ast::{JavaKnownType, JavaType},
    dialect::JavaDialect,
    lower::scalar_literal,
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaCharValuesInput {
    Type,
    Value(char),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaCharValues;

impl sealed::JavaCapabilityMapping for JavaCharValues {}
impl JavaCapabilityMapping for JavaCharValues {
    type Plan = mapping_plan::Plan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        mapping_plan::select(input)
    }
}

impl CapabilityMapping<JavaDialect> for JavaCharValues {
    type Capability = CharValues;
    type Context = ();
    type Input = JavaCharValuesInput;
    type Output = JavaValueNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaCharValuesInput::Type => {
                JavaValueNode::Type(JavaType::known(JavaKnownType::RuntimeScalar))
            }
            JavaCharValuesInput::Value(value) => {
                JavaValueNode::Expression(Box::new(scalar_literal(value)))
            }
        })
    }
}
