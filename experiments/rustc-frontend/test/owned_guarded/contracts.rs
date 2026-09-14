//! Branch/path certificates cannot be fabricated or erased into whole bodies.
#[cfg(owned_guard_private_body)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::guarded::GuardedOwnedBody<'_>) {
    let (guard, paths) = proof.into_parts();
    let _ = crate::owned_linear::multiple::guarded::GuardedOwnedBody { guard, paths };
}
#[cfg(owned_guard_private_guard)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::guarded::GuardEvidence<'_>) {
    let branch = proof.branch();
    let condition = proof.condition();
    let parameter = proof.parameter();
    let location = proof.location();
    let _ = crate::owned_linear::multiple::guarded::GuardEvidence {
        branch,
        condition,
        parameter,
        location,
    };
}
#[cfg(owned_guard_private_path)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::guarded::GuardedPath<'_>) {
    let outcome = proof.outcome();
    let scopes = proof.scopes().clone();
    let read = proof.scalar_read();
    let drops = proof.drop_order().to_vec();
    let returning = proof.returning();
    let chains = proof.into_chains();
    let _ = crate::owned_linear::multiple::guarded::GuardedPath {
        outcome,
        chains,
        scopes,
        read,
        drops,
        returning,
    };
}
#[cfg(owned_guard_not_whole)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::guarded::GuardedPath<'_>) {
    let _: crate::owned_linear::multiple::MultipleOwnedBody<'_> = proof;
}
