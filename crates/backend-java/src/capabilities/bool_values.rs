//! Java mapping for `BoolValues`.

use portable_build::{BoolValues, CapabilityMapping};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{ast::JavaExpr, dialect::JavaDialect, lower::bool_literal};

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaBoolValues;

impl sealed::JavaCapabilityMapping for JavaBoolValues {}
impl JavaCapabilityMapping for JavaBoolValues {}

impl CapabilityMapping<JavaDialect> for JavaBoolValues {
    type Capability = BoolValues;
    type Context = ();
    type Input = bool;
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(bool_literal(input))
    }
}
