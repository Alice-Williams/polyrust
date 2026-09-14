//! Neither the return wrapper nor its exit evidence has public construction.
#[cfg(owned_return_private_body)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::returns::ReturningOwnedBody<'_>) {
    let (body, exit) = proof.into_parts();
    let _ = crate::owned_linear::multiple::returns::ReturningOwnedBody { body, exit };
}
#[cfg(owned_return_private_exit)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::returns::ReturnEvidence<'_>) {
    let expression = proof.expression();
    let value = proof.value();
    let scope = proof.scope();
    let returning = proof.location();
    let _ = crate::owned_linear::multiple::returns::ReturnEvidence {
        expression,
        value,
        scope,
        returning,
    };
}
