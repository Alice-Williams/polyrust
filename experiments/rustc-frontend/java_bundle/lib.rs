//! Bounded descriptive bundles of exact certified Java owners. No filesystem I/O.
#![forbid(unsafe_code)]
mod budget;
mod bundle;
mod constant_exports;
mod constant_imports;
mod constants;
mod json;
mod manifest;
mod projection;
mod serialization;

pub use bundle::{BundleOutput, PreparedBundle};
use portable_backend_java::dialect::JavaDependencyApi;

/// The compiler adapter supplies authenticated owners; keys are descriptive only.
#[derive(Clone, Copy)]
pub struct Owner<'a> {
    pub key: &'a str,
    pub api: &'a JavaDependencyApi,
}

#[cfg(test)]
mod constant_fixture;
#[cfg(test)]
mod constant_tests;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod inventory_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod alias_fixture;
#[cfg(test)]
mod alias_tests;
