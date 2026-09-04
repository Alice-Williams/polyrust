//! Java mapping for the complete `Functions` capability.

use portable_build::{CapabilityMapping, Functions};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaExpr, JavaMethod},
    dialect::JavaDialect,
};

#[doc(hidden)]
pub enum JavaFunctionsNode {
    Declaration(JavaMethod),
    Expression(JavaExpr),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaFunctions;

impl sealed::JavaCapabilityMapping for JavaFunctions {}
impl JavaCapabilityMapping for JavaFunctions {}

impl CapabilityMapping<JavaDialect> for JavaFunctions {
    type Capability = Functions;
    type Context = ();
    type Input = JavaFunctionsNode;
    type Output = JavaFunctionsNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(input)
    }
}
