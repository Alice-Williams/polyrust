//! Java mapping for the complete `UnitValues` capability.

use portable_build::{CapabilityMapping, UnitValues};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{ast::JavaExpr, dialect::JavaDialect, lower::java_unit_value};

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaUnitValues;

impl sealed::JavaCapabilityMapping for JavaUnitValues {}
impl JavaCapabilityMapping for JavaUnitValues {}

impl CapabilityMapping<JavaDialect> for JavaUnitValues {
    type Capability = UnitValues;
    type Context = ();
    type Input = ();
    type Output = JavaExpr;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        (): Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(java_unit_value())
    }
}
