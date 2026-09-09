//! Checked numeric facts; tree evaluation retains the original source AST.
mod conversion;
mod integer_operations;
mod known_values;
mod numeric;
mod tree;

pub(super) use numeric::{CInteger, CNumber};
pub(super) use tree::evaluate;

#[cfg(test)]
#[path = "../tests/constant_numbers.rs"]
mod numbers_test;

#[cfg(test)]
#[path = "../tests/constant_conversions.rs"]
mod conversions_test;
