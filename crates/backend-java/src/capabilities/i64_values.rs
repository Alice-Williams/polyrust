//! Java mapping for `I64Values`.

use portable_build::{CapabilityMapping, I64Values};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{ast::JavaExpr, dialect::JavaDialect, lower::i64_literal};

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaI64Values;

impl sealed::JavaCapabilityMapping for JavaI64Values {}
impl JavaCapabilityMapping for JavaI64Values {}

impl CapabilityMapping<JavaDialect> for JavaI64Values {
    type Capability = I64Values;
    type Context = ();
    type Input = i64;
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(i64_literal(input))
    }
}
