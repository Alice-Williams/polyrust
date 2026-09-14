//! Deliberate internal adapter defects; none are compiled into production.
#[cfg(java_graph_wrong_owner)]
pub(crate) fn owner(
    original: &portable_backend_java::dialect::JavaDependencyApi,
    dependencies: &crate::source_check::CheckedDependencies<'_, '_, super::CheckedCrate>,
) -> portable_backend_java::dialect::JavaDependencyApi {
    dependencies
        .values()
        .map(|item| &item.api)
        .find(|api| api.root() != original.root())
        .expect("mutation fixture has a different authenticated owner")
        .clone()
}

#[cfg(java_graph_wrong_declaration)]
pub(crate) fn declaration(
    owner: &portable_backend_java::dialect::JavaDependencyApi,
    original: portable_backend_java::dialect::JavaDependencyFunction,
) -> portable_backend_java::dialect::JavaDependencyFunction {
    owner
        .functions()
        .find(|function| function.declaration() != original.declaration())
        .expect("mutation fixture has a second public declaration")
        .clone()
}

#[cfg(java_graph_wrong_signature)]
pub(crate) fn signature(
    mut original: portable_backend_java::ast::JavaMethodSignature,
) -> portable_backend_java::ast::JavaMethodSignature {
    use portable_backend_java::ast::{JavaPrimitive, JavaType};
    original.result = match original.result {
        JavaType::Primitive(JavaPrimitive::Int) => JavaType::primitive(JavaPrimitive::Boolean),
        _ => JavaType::primitive(JavaPrimitive::Int),
    };
    original
}
