//! Java mapping for the complete `UnitValues` capability.

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
pub enum JavaUnitValuesInput {
    Type,
    Value,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaUnitValues;

impl sealed::JavaCapabilityMapping for JavaUnitValues {}
impl JavaCapabilityMapping for JavaUnitValues {}

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
