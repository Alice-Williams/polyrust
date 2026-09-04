//! Java mapping for `CharValues`.

use portable_build::{CapabilityMapping, CharValues};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{ast::JavaExpr, dialect::JavaDialect, lower::scalar_literal};

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaCharValues;

impl sealed::JavaCapabilityMapping for JavaCharValues {}
impl JavaCapabilityMapping for JavaCharValues {}

impl CapabilityMapping<JavaDialect> for JavaCharValues {
    type Capability = CharValues;
    type Context = ();
    type Input = char;
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(scalar_literal(input))
    }
}
