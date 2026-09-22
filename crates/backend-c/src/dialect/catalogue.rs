//! One authority for standard identities, signatures and operand obligations.

mod constant_spellings;
mod known_calls;
mod operands;
mod signatures;
mod type_headers;

pub use known_calls::{CHeader, CKnownCall, CKnownCallForm, CSystemLibrary};
pub use operands::CKnownOperands;
