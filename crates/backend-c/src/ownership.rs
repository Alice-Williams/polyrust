//! Private safety analyses; diagnostic success cannot render a C package.

mod constants;
mod context_facts;
mod errors;
mod layout;
mod loops;
mod numeric_flow;
mod package_constants;
mod ranges;
mod sequencing;
mod storage;

pub use errors::CSafetyError;
