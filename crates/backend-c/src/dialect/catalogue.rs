//! One authority for standard identities, signatures and operand obligations.

mod known_calls;
mod operands;
mod signatures;

pub use known_calls::{CHeader, CKnownCall, CKnownCallForm, CSystemLibrary};
pub use operands::CKnownOperands;
