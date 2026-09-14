//! Private certificates cannot be fabricated or erased into linear evidence.
#[cfg(selection_private_body)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::selection::SelectedOwnedBody<'_>) {
    let _ = crate::owned_linear::multiple::selection::SelectedOwnedBody { ..proof };
}
#[cfg(selection_private_path)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::selection::SelectionPath<'_>) {
    let _ = crate::owned_linear::multiple::selection::SelectionPath { ..proof };
}
#[cfg(selection_private_flag)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::selection::Decision) {
    let _ = crate::owned_linear::multiple::selection::Decision { ..proof };
}
#[cfg(selection_not_whole)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::selection::SelectionPath<'_>) {
    let _: crate::owned_linear::multiple::MultipleOwnedBody<'_> = proof;
}
#[cfg(selection_not_guarded)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::selection::SelectedOwnedBody<'_>) {
    let _: crate::owned_linear::multiple::guarded::GuardedOwnedBody<'_> = proof;
}
