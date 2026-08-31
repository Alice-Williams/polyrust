//! Deterministic reference evaluation for checked portable programs.
//!
//! The public entry points accept [`portable_check::v0::CheckedProgram`], never
//! unchecked IR. Runtime values are the exact v0 Core value algebra, while
//! errors and canonical JSON form a target-independent conformance protocol.

#![forbid(unsafe_code)]

mod canonical;
mod evaluator;

pub use canonical::{
    CanonicalDecodeError, decode_canonical_error, decode_canonical_outcome, decode_canonical_value,
    encode_canonical_error, encode_canonical_outcome, encode_canonical_value,
};
pub use evaluator::{
    EvaluationError, EvaluationLimits, EvaluationOutcome, Evaluator, PortableTestResult,
};
pub use portable_ir::v0::Value;

#[cfg(test)]
mod tests;
