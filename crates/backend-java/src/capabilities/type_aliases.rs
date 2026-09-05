//! Java mapping for the complete `TypeAliases` capability.

use portable_build::{CapabilityMapping, TypeAliases};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{JavaIdentifier, JavaType},
    dialect::JavaDialect,
    lower::identifier,
};

#[doc(hidden)]
pub struct JavaTypeAliasInput {
    pub(crate) name: String,
    pub(crate) target: JavaType,
}

/// Java erases portable transparent aliases after validating their name and target.
#[doc(hidden)]
pub struct JavaErasedTypeAlias {
    pub(crate) _name: JavaIdentifier,
    pub(crate) _target: JavaType,
}

impl sealed::JavaMappingOutput for JavaErasedTypeAlias {}
impl super::support::JavaMappingOutput for JavaErasedTypeAlias {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaTypeAliases;

impl sealed::JavaCapabilityMapping for JavaTypeAliases {}
impl JavaCapabilityMapping for JavaTypeAliases {}

impl CapabilityMapping<JavaDialect> for JavaTypeAliases {
    type Capability = TypeAliases;
    type Context = ();
    type Input = JavaTypeAliasInput;
    type Output = JavaErasedTypeAlias;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(JavaErasedTypeAlias {
            _name: identifier(&input.name),
            _target: input.target,
        })
    }
}
