//! Java mapping for the complete `LocalBindings` capability.

use portable_build::{CapabilityMapping, LocalBindings};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaExpr, JavaLocalFinality, JavaStmt, JavaType},
    dialect::JavaDialect,
    lower::identifier,
};

#[doc(hidden)]
pub struct JavaLocalBindingInput {
    pub(crate) name: String,
    pub(crate) ty: JavaType,
    pub(crate) value: JavaExpr,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaLocalBindings;

impl sealed::JavaCapabilityMapping for JavaLocalBindings {}
impl JavaCapabilityMapping for JavaLocalBindings {}

impl CapabilityMapping<JavaDialect> for JavaLocalBindings {
    type Capability = LocalBindings;
    type Context = ();
    type Input = JavaLocalBindingInput;
    type Output = JavaStmt;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: input.ty,
            name: identifier(&input.name),
            value: Some(input.value),
        })
    }
}
