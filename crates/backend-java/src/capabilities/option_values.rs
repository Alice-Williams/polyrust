//! Java mapping for `OptionValues`.

use portable_build::{CapabilityMapping, OptionValues};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaExpr, JavaType},
    dialect::{JavaDialect, JavaRuntimeCallable},
    lower::runtime_call,
};

#[doc(hidden)]
pub enum JavaOptionInput {
    None {
        result: JavaType,
    },
    Some {
        value: Box<JavaExpr>,
        result: JavaType,
    },
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaOptionValues;

impl sealed::JavaCapabilityMapping for JavaOptionValues {}
impl JavaCapabilityMapping for JavaOptionValues {}

impl CapabilityMapping<JavaDialect> for JavaOptionValues {
    type Capability = OptionValues;
    type Context = ();
    type Input = JavaOptionInput;
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let (callable, arguments, result) = match input {
            JavaOptionInput::None { result } => (JavaRuntimeCallable::OptionNone, vec![], result),
            JavaOptionInput::Some { value, result } => {
                (JavaRuntimeCallable::OptionSome, vec![*value], result)
            }
        };
        Ok(runtime_call(callable, arguments, result))
    }
}
