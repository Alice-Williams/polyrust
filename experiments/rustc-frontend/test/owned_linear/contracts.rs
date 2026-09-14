//! External consumers cannot fabricate correspondence evidence.
#[cfg(owned_linear_private)]
#[allow(dead_code)]
fn fabricated<'tcx>(proof: crate::owned_linear::LinearOwnedBody<'tcx>) {
    let parameter = proof.parameter();
    let bindings = proof.bindings().to_vec();
    let scope = proof.scope();
    let moves = proof.moves().to_vec();
    let scalar_read = proof.scalar_read();
    let drop = proof.drop_location();
    let _ = crate::owned_linear::LinearOwnedBody {
        constructor: proof.into_construction(),
        parameter,
        bindings,
        scope,
        moves,
        scalar_read,
        drop,
    };
}
