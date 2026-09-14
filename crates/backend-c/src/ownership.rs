//! Private safety analyses; diagnostic success cannot render a C package.

mod constants;
mod context_facts;
mod errors;
pub(crate) mod layout;
mod loops;
mod numeric_flow;
mod package_constants;
mod paths;
mod ranges;
mod scalar_calls;
mod sequencing;
mod storage;

pub use errors::CSafetyError;

pub(crate) fn closed_scalar_functions(
    registry: &crate::ast::CRegistry,
    files: &[crate::ast::CSourceFile],
) -> Result<std::collections::BTreeSet<crate::ast::CFunctionRef>, crate::ast::CRegistryError> {
    Ok(scalar_calls::ScalarCalls::derive_registered(registry, files)?.into_functions())
}
