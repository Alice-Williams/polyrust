//! Java mapping for the complete `Records` capability.

use portable_build::{CapabilityMapping, Records};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaExpr, JavaTypeDeclaration},
    dialect::JavaDialect,
};

#[doc(hidden)]
pub enum JavaRecordsNode {
    Declaration(JavaTypeDeclaration),
    Expression(JavaExpr),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaRecords;

impl sealed::JavaCapabilityMapping for JavaRecords {}
impl JavaCapabilityMapping for JavaRecords {}

impl CapabilityMapping<JavaDialect> for JavaRecords {
    type Capability = Records;
    type Context = ();
    type Input = JavaRecordsNode;
    type Output = JavaRecordsNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(input)
    }
}
