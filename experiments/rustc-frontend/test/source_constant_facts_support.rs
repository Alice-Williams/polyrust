//! Metadata-only probes link the real evaluator, without concrete backends.
#[path = "../src/source_capabilities/constant_evaluation.rs"]
mod constant_evaluation;
#[path = "../src/source_capabilities/constant_values.rs"]
mod constant_values;

pub(crate) use constant_evaluation::original_value as original_constant_value;
use constant_values::ScalarConstantValue;
