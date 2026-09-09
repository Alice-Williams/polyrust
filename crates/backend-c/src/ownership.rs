//! Private safety analyses; diagnostic success cannot render a C package.

mod constants;
mod context_facts;
mod errors;
mod layout;
mod package_constants;
mod sequencing;

pub use errors::CSafetyError;
