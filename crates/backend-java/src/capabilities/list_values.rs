//! Java mapping for `ListValues`.

use portable_build::{CapabilityMapping, ListValues};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaExpr, JavaType},
    dialect::{JavaDialect, JavaKnownCallable},
    lower::known_generic_call,
};

#[doc(hidden)]
pub struct JavaListInput {
    pub(crate) elements: Vec<JavaExpr>,
    pub(crate) result: JavaType,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaListValues;

impl sealed::JavaCapabilityMapping for JavaListValues {}
impl JavaCapabilityMapping for JavaListValues {}

impl CapabilityMapping<JavaDialect> for JavaListValues {
    type Capability = ListValues;
    type Context = ();
    type Input = JavaListInput;
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(known_generic_call(
            JavaKnownCallable::ListOf,
            input.elements,
            input.result,
        ))
    }
}
