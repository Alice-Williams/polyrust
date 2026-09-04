//! Java mapping for `F64Values`.

use portable_build::{CapabilityMapping, F64Values};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{ast::JavaExpr, dialect::JavaDialect, lower::f64_literal};

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaF64Values;

impl sealed::JavaCapabilityMapping for JavaF64Values {}
impl JavaCapabilityMapping for JavaF64Values {}

impl CapabilityMapping<JavaDialect> for JavaF64Values {
    type Capability = F64Values;
    type Context = ();
    type Input = u64;
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(f64_literal(input))
    }
}
