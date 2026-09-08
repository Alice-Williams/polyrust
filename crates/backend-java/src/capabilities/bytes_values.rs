//! Java mapping for `BytesValues`.

mod mapping_plan;

use portable_build::{BytesValues, CapabilityMapping};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, JavaValueNode, sealed};
use crate::{
    ast::{JavaKnownType, JavaPrimitive, JavaType},
    dialect::{JavaDialect, JavaKnownCallable, JavaRuntimeCallable},
    lower::{i32_literal, known_generic_call, runtime_call},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaBytesInput {
    Type,
    Value { values: Vec<u8>, result: JavaType },
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaBytesValues;

impl sealed::JavaCapabilityMapping for JavaBytesValues {}
impl JavaCapabilityMapping for JavaBytesValues {
    type Plan = mapping_plan::Plan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        mapping_plan::select(input)
    }
}

impl CapabilityMapping<JavaDialect> for JavaBytesValues {
    type Capability = BytesValues;
    type Context = ();
    type Input = JavaBytesInput;
    type Output = JavaValueNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaBytesInput::Type => {
                JavaValueNode::Type(JavaType::known(JavaKnownType::RuntimeBytes))
            }
            JavaBytesInput::Value { values, result } => {
                let list = JavaType::generic(
                    JavaKnownType::List,
                    vec![JavaType::Boxed(JavaPrimitive::Int)],
                );
                let elements = values
                    .into_iter()
                    .map(|value| i32_literal(i32::from(value)))
                    .collect();
                JavaValueNode::Expression(Box::new(runtime_call(
                    JavaRuntimeCallable::BytesOf,
                    vec![known_generic_call(
                        JavaKnownCallable::ListOf,
                        elements,
                        list,
                    )],
                    result,
                )))
            }
        })
    }
}
