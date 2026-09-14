//! External consumers cannot fabricate correspondence evidence.
#[cfg(owned_scope_private)]
#[allow(dead_code)]
fn fabricated_scope<'tcx>(proof: crate::owned_linear::LinearOwnedBody<'tcx>) {
    let _ = crate::owned_linear::ScopeEvidence {
        blocks: Vec::new(),
        bindings: Vec::new(),
        read: proof.scope(),
        drop: proof.scope(),
    };
}
#[cfg(owned_linear_private)]
#[allow(dead_code)]
fn fabricated<'tcx>(proof: crate::owned_linear::LinearOwnedBody<'tcx>) {
    let parameter = proof.parameter();
    let bindings = proof.bindings().to_vec();
    let scope = proof.scope();
    let scopes = proof.scopes().clone();
    let moves = proof.moves().to_vec();
    let scalar_read = proof.scalar_read();
    let drop = proof.drop_location();
    let _ = crate::owned_linear::LinearOwnedBody {
        constructor: proof.into_construction(),
        parameter,
        bindings,
        scope,
        scopes,
        moves,
        scalar_read,
        drop,
    };
}
