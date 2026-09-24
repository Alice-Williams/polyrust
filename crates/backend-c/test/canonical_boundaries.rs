//! Actual same-core type owners reach the unchanged dependency certificate limit.
#[path = "canonical_dependencies/fixture.rs"]
mod fixture;
#[path = "canonical_dependencies/source.rs"]
mod source;

#[test]
fn canonical_owner_closure_accepts_1024_and_rejects_exactly_one_more() {
    let owners: Vec<_> = (0..=1024).map(|index| fixture::api(index * 32)).collect();
    let proofs: Vec<_> = owners
        .iter()
        .map(|api| api.structs().next().unwrap().clone())
        .collect();
    let accepted = source::source(source::root(200), &proofs[..1024], &[], false).unwrap();
    assert_eq!(accepted.dependencies().unwrap().len(), 1024);
    assert!(
        accepted.structs().next().is_none(),
        "imports are deliberately unused"
    );
    assert!(accepted.functions().next().unwrap().stack_bound_bytes() > 0);
    let error = source::source(source::root(201), &proofs, &[], false).unwrap_err();
    assert!(
        error.contains("C dependency certificate budget exceeded"),
        "{error}"
    );
}
