//! Stable structured diagnostics with terminal and JSON renderers.

#![forbid(unsafe_code)]

pub mod code;
pub mod model;
pub mod render;

pub use code::*;
pub use model::*;
pub use portable_ir::v0::{FileSpan, LogicalSource, SourceRef};
pub use render::*;

#[cfg(test)]
mod tests;
