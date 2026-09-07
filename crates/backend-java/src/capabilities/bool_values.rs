//! Java mapping for `BoolValues`.

use portable_build::{BoolValues, CapabilityMapping};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, JavaValueNode, sealed};
use crate::{
    ast::{JavaPrimitive, JavaType},
    dialect::JavaDialect,
    lower::bool_literal,
};

#[doc(hidden)]
pub enum JavaBoolValuesInput {
    Type,
    Value(bool),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaBoolValues;

impl sealed::JavaCapabilityMapping for JavaBoolValues {}
impl JavaCapabilityMapping for JavaBoolValues {}

impl CapabilityMapping<JavaDialect> for JavaBoolValues {
    type Capability = BoolValues;
    type Context = ();
    type Input = JavaBoolValuesInput;
    type Output = JavaValueNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaBoolValuesInput::Type => {
                JavaValueNode::Type(JavaType::primitive(JavaPrimitive::Boolean))
            }
            JavaBoolValuesInput::Value(value) => {
                JavaValueNode::Expression(Box::new(bool_literal(value)))
            }
        })
    }
}
