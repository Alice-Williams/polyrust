//! Java mapping for `BytesValues`.

use portable_build::{BytesValues, CapabilityMapping};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaExpr, JavaKnownType, JavaPrimitive, JavaType},
    dialect::{JavaDialect, JavaKnownCallable, JavaRuntimeCallable},
    lower::{i32_literal, known_generic_call, runtime_call},
};

#[doc(hidden)]
pub struct JavaBytesInput {
    pub(crate) values: Vec<u8>,
    pub(crate) result: JavaType,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaBytesValues;

impl sealed::JavaCapabilityMapping for JavaBytesValues {}
impl JavaCapabilityMapping for JavaBytesValues {}

impl CapabilityMapping<JavaDialect> for JavaBytesValues {
    type Capability = BytesValues;
    type Context = ();
    type Input = JavaBytesInput;
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let list = JavaType::generic(
            JavaKnownType::List,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        );
        let elements = input
            .values
            .into_iter()
            .map(|value| i32_literal(i32::from(value)))
            .collect();
        Ok(runtime_call(
            JavaRuntimeCallable::BytesOf,
            vec![known_generic_call(
                JavaKnownCallable::ListOf,
                elements,
                list,
            )],
            input.result,
        ))
    }
}
