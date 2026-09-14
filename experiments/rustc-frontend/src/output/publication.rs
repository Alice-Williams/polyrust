//! C-specific payload counts and flat filename policy.
use portable_directory_publication::{PathPolicy, publish};
use std::path::Path;

pub(super) fn new_directory(path: &Path, files: &[(String, String)]) -> Result<(), String> {
    if files.len() != 3 {
        return Err("package publication requires header, source and manifest".into());
    }
    publish(path, files, PathPolicy::Flat)
}

pub(super) fn bundle_directory(
    path: &Path,
    files: &[(String, String)],
    members: usize,
) -> Result<(), String> {
    if members == 0 || members > 1024 || files.len() != 3 * members + 1 {
        return Err("bundle publication requires exactly three files per member plus index".into());
    }
    publish(path, files, PathPolicy::Flat)
}
