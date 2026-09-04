//! Java mapping for `I32Values`.

use portable_build::{CapabilityMapping, I32Values};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{ast::JavaExpr, dialect::JavaDialect, lower::i32_literal};

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaI32Values;

impl sealed::JavaCapabilityMapping for JavaI32Values {}
impl JavaCapabilityMapping for JavaI32Values {}

impl CapabilityMapping<JavaDialect> for JavaI32Values {
    type Capability = I32Values;
    type Context = ();
    type Input = i32;
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(i32_literal(input))
    }
}
