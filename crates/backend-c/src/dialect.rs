//! Closed C library identities. Shared linking is a separate migration stage.

mod catalogue;

pub use catalogue::{CHeader, CKnownCall, CKnownCallForm, CKnownOperands, CSystemLibrary};
