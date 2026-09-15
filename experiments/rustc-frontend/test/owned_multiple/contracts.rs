//! Public consumers cannot assemble a body or individual owner-chain evidence.
#[cfg(owned_multiple_private_body)]
#[allow(dead_code)]
fn fake_body<'tcx>(proof: crate::owned_linear::multiple::MultipleOwnedBody<'tcx>) {
    let scopes = proof.scopes().clone();
    let read = proof.scalar_read();
    let drops = proof.drop_order().to_vec();
    let returning = proof.returning();
    let chains = proof.into_chains();
    let _ = crate::owned_linear::multiple::MultipleOwnedBody {
        chains,
        scopes,
        read,
        drops,
        returning,
    };
}
#[cfg(owned_multiple_private_chain)]
#[allow(dead_code)]
fn fake_chain<'tcx>(chain: crate::owned_linear::multiple::ChainEvidence<'tcx>) {
    let parameter = chain.parameter();
    let bindings = chain.bindings().to_vec();
    let moves = chain.moves().to_vec();
    let drop = chain.drop_location();
    let drop_scope = chain.drop_scope();
    let constructor = chain.into_construction();
    let _ = crate::owned_linear::multiple::ChainEvidence {
        constructor,
        parameter,
        bindings,
        moves,
        drop,
        drop_scope,
    };
}
