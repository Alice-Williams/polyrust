//! Deliberate typed join corruptions, compiled only into negative-test binaries.

#[cfg(c_graph_wrong_owner)]
pub(crate) fn owner<'a>(
    expected: &'a portable_backend_c::dialect::CDependencyApi,
    dependencies: &crate::source_check::CheckedDependencies<'a, '_, crate::c_graph::CheckedCrate>,
) -> &'a portable_backend_c::dialect::CDependencyApi {
    dependencies
        .values()
        .map(|item| item.api())
        .find(|candidate| candidate.root() != expected.root())
        .expect("owner mutation fixture requires two distinct checked owners")
}

#[cfg(c_graph_wrong_declaration)]
pub(crate) fn declaration(
    owner: &portable_backend_c::dialect::CDependencyApi,
    expected: portable_backend_c::dialect::CDependencyFunction,
) -> portable_backend_c::dialect::CDependencyFunction {
    owner
        .functions()
        .find(|candidate| {
            candidate.declaration() != expected.declaration()
                && candidate.signature() == expected.signature()
        })
        .expect("declaration mutation fixture requires another public identical signature")
        .clone()
}

#[cfg(c_graph_wrong_signature)]
pub(crate) fn signature(
    expected: portable_backend_c::ast::CFunctionType,
) -> portable_backend_c::ast::CFunctionType {
    use portable_backend_c::ast::*;
    let replacement = CFunctionType::new(
        CReturnType::Value(CReturnValue::new(CObjectType::scalar(CScalarType::Bool)).unwrap()),
        vec![],
    );
    assert_ne!(expected, replacement);
    replacement
}
