#![forbid(unsafe_code)]

//! Canonical, verified, target-neutral semantic IR.

mod ids;
mod lower;
mod model;
mod verify;

pub use ids::*;
pub use lower::{CanonicalCoreLowerer, lower_checked};
pub use model::*;
pub use verify::{constant_expression_type, verify_core};

#[cfg(test)]
mod tests;
