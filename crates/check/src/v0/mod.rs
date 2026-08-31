//! Checked representation and validation entry point for portable IR v0.

mod checked;
mod checker;

pub use checked::*;
pub use checker::check_program;

#[cfg(test)]
mod tests;
