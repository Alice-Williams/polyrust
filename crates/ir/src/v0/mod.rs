//! Versioned, target-independent unchecked IR.
//!
//! This module represents portable programs before name resolution or type
//! checking. It deliberately contains no Rust, Go, Python, or TypeScript syntax.
//!
//! # Example
//!
//! ```
//! use portable_ir::v0::{
//!     Declaration, DeclarationHeader, Document, IrVersion, Module, NodeId,
//!     NodeMeta, RecordDeclaration, SourceRef, Visibility, to_canonical_json,
//! };
//!
//! let source = SourceRef::logical(["module(example)", "record(User)"]);
//! let record = RecordDeclaration {
//!     header: DeclarationHeader {
//!         node: NodeMeta::new(NodeId::new(1), source),
//!         name: "User".into(),
//!         visibility: Visibility::Public,
//!         documentation: vec!["A portable user value.".into()],
//!     },
//!     fields: vec![],
//! };
//! let document = Document::new(
//!     IrVersion::CURRENT,
//!     Module {
//!         name: "example".into(),
//!         declarations: vec![Declaration::Record(record)],
//!     },
//! );
//!
//! let json = to_canonical_json(&document).expect("valid IR serializes");
//! assert!(json.starts_with(br#"{"ir_version":"0.1.0""#));
//! ```

mod common;
mod declaration;
mod expression;
mod json;
mod validate;
mod visit;

pub use common::*;
pub use declaration::*;
pub use expression::*;
pub use json::*;
pub use validate::*;
pub use visit::*;

#[cfg(test)]
mod tests;
