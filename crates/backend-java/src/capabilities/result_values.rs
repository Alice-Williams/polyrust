//! Java mapping for `ResultValues`.

use portable_build::{CapabilityMapping, ResultValues};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaExpr, JavaType},
    dialect::{JavaDialect, JavaRuntimeCallable},
    lower::runtime_call,
};

#[doc(hidden)]
pub enum JavaResultInput {
    Ok { value: JavaExpr, result: JavaType },
    Err { value: JavaExpr, result: JavaType },
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaResultValues;

impl sealed::JavaCapabilityMapping for JavaResultValues {}
impl JavaCapabilityMapping for JavaResultValues {}

impl CapabilityMapping<JavaDialect> for JavaResultValues {
    type Capability = ResultValues;
    type Context = ();
    type Input = JavaResultInput;
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let (callable, value, result) = match input {
            JavaResultInput::Ok { value, result } => {
                (JavaRuntimeCallable::ValueResultOk, value, result)
            }
            JavaResultInput::Err { value, result } => {
                (JavaRuntimeCallable::ValueResultErr, value, result)
            }
        };
        Ok(runtime_call(callable, vec![value], result))
    }
}
