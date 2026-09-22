//! Shared test-only primitive arithmetic packages and dependency authority.
pub(super) use crate::tests::source_dependency_fixture as f;
pub(super) use crate::{ast::*, dialect::*};
pub(super) use portable_codegen::*;

#[path = "wrapping_integer_fixture.rs"]
pub(super) mod fixture;
