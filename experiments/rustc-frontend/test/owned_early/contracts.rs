//! Early-return bodies are private and cannot pretend to be the if/else grammar.
#[cfg(owned_early_private)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::early::EarlyOwnedBody<'_>) {
    let (guard, paths) = proof.into_parts();
    let _ = crate::owned_linear::multiple::early::EarlyOwnedBody { guard, paths };
}
#[cfg(owned_early_not_if_else)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::early::EarlyOwnedBody<'_>) {
    let _: crate::owned_linear::multiple::guarded::GuardedOwnedBody<'_> = proof;
}
