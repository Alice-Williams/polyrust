//! Java mapping for the complete `UnitValues` capability.

mod mapping_plan;

use portable_build::{CapabilityMapping, UnitValues};
use portable_diagnostics::Diagnostic;

use super::support::JavaValueNode;
use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaKnownType, JavaType},
    dialect::JavaDialect,
    lower::java_unit_value,
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaUnitValuesInput {
    Type,
    Value,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaUnitValues;

impl sealed::JavaCapabilityMapping for JavaUnitValues {}
impl JavaCapabilityMapping for JavaUnitValues {
    type Plan = mapping_plan::Plan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        mapping_plan::select(input)
    }
}

impl CapabilityMapping<JavaDialect> for JavaUnitValues {
    type Capability = UnitValues;
    type Context = ();
    type Input = JavaUnitValuesInput;
    type Output = JavaValueNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaUnitValuesInput::Type => {
                JavaValueNode::Type(JavaType::known(JavaKnownType::RuntimeUnit))
            }
            JavaUnitValuesInput::Value => JavaValueNode::Expression(Box::new(java_unit_value())),
        })
    }
}
