//! Descriptive package placement, distinct from a package's opaque authority.

use super::{RustCanonicalInstanceKey, RustDeclarationId};

/// A real source crate or an explicitly generated canonical instance package.
///
/// `P` is a backend-owned closed representation-profile type. Generic shared
/// metadata does not decide which representations a backend supports. Backends
/// must verify the full descriptor from their immutable AST before certification;
/// constructing this enum cannot attach provenance to a certified package.
/// This generic container is allocation-free only when the selected `P` is;
/// unlike the key/facts, it makes no size promise for arbitrary caller types.
///
/// Identity excludes consuming crates and traversal order. Distinct instances
/// may have the same actual core anchor without becoming one generated owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetPackageOwner<P> {
    SourceCrate(RustDeclarationId),
    CanonicalInstance {
        instance: RustCanonicalInstanceKey,
        profile: P,
    },
}
