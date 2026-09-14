//! Bounded private staging and Linux atomic no-replace publication.
//!
//! This is a filesystem transaction, not an AST or certificate authority.
//! Callers must first certify and reserve their complete language inventory.
#![forbid(unsafe_code)]

mod paths;
mod transaction;

pub use paths::{PathPolicy, TreeLimits};
use std::path::Path;

/// Publish the complete payload or leave the destination absent/unchanged.
/// The parent and declared native helper are trusted local build infrastructure.
/// Crash durability and hostile same-user mutation of staging are not promised.
pub fn publish(
    destination: &Path,
    files: &[(String, String)],
    policy: PathPolicy,
) -> Result<(), String> {
    let directories = paths::validate(files, policy)?;
    transaction::publish(destination, files, &directories)
}

#[cfg(test)]
mod tests;
