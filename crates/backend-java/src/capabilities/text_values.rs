//! Java mapping for `TextValues`.

use portable_build::{CapabilityMapping, TextValues};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{ast::JavaExpr, dialect::JavaDialect, lower::string_literal};

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaTextValues;

impl sealed::JavaCapabilityMapping for JavaTextValues {}
impl JavaCapabilityMapping for JavaTextValues {}

impl CapabilityMapping<JavaDialect> for JavaTextValues {
    type Capability = TextValues;
    type Context = ();
    type Input = String;
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(string_literal(&input))
    }
}
