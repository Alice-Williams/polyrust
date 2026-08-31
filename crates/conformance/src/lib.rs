//! Differential conformance between the reference evaluator and generated
//! native target programs.
//!
//! Harness behavior arrives in M14. This outer-layer crate may compose concrete
//! backends; semantic-core crates may not depend on it.

#![forbid(unsafe_code)]
