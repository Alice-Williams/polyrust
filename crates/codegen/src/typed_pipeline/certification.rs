//! One sealed syntax-then-resource admission path for all rendering clients.

use std::marker::PhantomData;

use portable_diagnostics::Diagnostic;

use super::{LinkedPackage, TargetDialect, TypedGenerationError, TypedPipelineStage};

/// An opaque capability proving that the language-owned post-link checker
/// and resource admission accepted the exact package presented to rendering.
///
/// It deliberately implements neither `Deserialize` nor mutable AST access:
///
/// ~~~compile_fail
/// use portable_codegen::{RenderReadyPackage, TargetDialect};
///
/// fn cannot_deserialize<D: TargetDialect>(json: &str) -> RenderReadyPackage<D> {
///     serde_json::from_str(json).unwrap()
/// }
/// ~~~
///
/// ~~~compile_fail
/// use portable_codegen::{RenderReadyPackage, TargetDialect};
///
/// fn cannot_mutate<D: TargetDialect>(
///     package: &mut RenderReadyPackage<D>,
///     replacement: D::Resolved,
/// ) {
///     *package.ast() = replacement;
/// }
/// ~~~
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderReadyPackage<D: TargetDialect> {
    ast: D::Resolved,
    dialect: PhantomData<fn() -> D>,
}

impl<D: TargetDialect> RenderReadyPackage<D> {
    pub fn ast(&self) -> &D::Resolved {
        &self.ast
    }

    fn new(ast: D::Resolved) -> Self {
        Self {
            ast,
            dialect: PhantomData,
        }
    }
}

pub fn certify_linked_package<D: TargetDialect>(
    dialect: &D,
    package: LinkedPackage<D>,
) -> Result<RenderReadyPackage<D>, Vec<Diagnostic>> {
    certify_with_stage(dialect, package).map_err(CertificationFailure::into_diagnostics)
}

/// Runs the language-owned post-link checker and returns the only public path
/// from a raw resolved value to the opaque rendering capability.
pub fn certify_resolved_package<D: TargetDialect>(
    dialect: &D,
    ast: D::Resolved,
) -> Result<RenderReadyPackage<D>, Vec<Diagnostic>> {
    certify_linked_package(dialect, LinkedPackage::new(ast))
}
#[derive(Debug)]
pub(super) enum CertificationFailure {
    Language(Vec<Diagnostic>),
    Resources(Vec<Diagnostic>),
}

impl CertificationFailure {
    fn into_diagnostics(self) -> Vec<Diagnostic> {
        match self {
            Self::Language(diagnostics) | Self::Resources(diagnostics) => diagnostics,
        }
    }

    pub(super) fn into_generation_error(self) -> TypedGenerationError {
        let (stage, diagnostics) = match self {
            Self::Language(diagnostics) => (
                TypedPipelineStage::RenderReadinessCertification,
                diagnostics,
            ),
            Self::Resources(diagnostics) => {
                (TypedPipelineStage::TargetResourceValidation, diagnostics)
            }
        };
        TypedGenerationError::phase(stage, diagnostics)
    }
}

/// Both checks consume the same immutable linked payload. No certificate exists
/// on either failure path, including calls outside the convenience compiler.
pub(super) fn certify_with_stage<D: TargetDialect>(
    dialect: &D,
    package: LinkedPackage<D>,
) -> Result<RenderReadyPackage<D>, CertificationFailure> {
    dialect
        .verify_resolved(package.ast())
        .map_err(CertificationFailure::Language)?;
    dialect
        .verify_resources(package.ast())
        .map_err(CertificationFailure::Resources)?;
    Ok(RenderReadyPackage::new(package.into_ast()))
}

#[cfg(test)]
#[path = "../tests/certification.rs"]
mod tests;
