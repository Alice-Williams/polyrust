//! Compiler metadata must not weaken target registry authentication.
use super::{
    CContextError, CDeclarationKey, CDeclarations, CExpressionError, CExpressions, CFileItem,
    CFileKey, CFileRef, CFileRole, CFunctionRef, CFunctionType, CGeneratedOrigin, CIdentifier,
    CLinkage, CObjectType, CRegistry, CRegistryError, CReturnType, CScalarType, CScopeRef,
    CStatements,
};
use portable_codegen::{
    RelativeOutputPath, RustDeclarationId, RustSourceLocation, RustSourceNode, RustSourceOrigin,
    RustVisibility,
};

fn key(node: RustSourceNode, name: &str) -> CDeclarationKey {
    let declaration = RustDeclarationId {
        crate_id: 1,
        definition_path_hash: 2,
    };
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::RustSource(std::sync::Arc::new(RustSourceOrigin {
            declaration,
            node,
            module: declaration,
            location: RustSourceLocation {
                file: "source.rs".into(),
                line: 1,
                column: 0,
            },
            visibility: RustVisibility::RestrictedTo(declaration),
            externally_reachable: false,
            documentation: vec!["source documentation, not code".into()],
            module_ancestors: [].into(),
            crate_exports: std::sync::Arc::new(portable_codegen::RustCrateExports {
                module_ancestries: std::collections::BTreeMap::new(),
                root: declaration,
                modules: Default::default(),
            }),
        })),
    }
}

fn fixture(role: CFileRole) -> (CRegistry, CFileRef, CFunctionRef, CScopeRef) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("generated.c").unwrap(),
            role,
        })
        .unwrap();
    let function = registry
        .register_function(
            &file,
            key(RustSourceNode::Declaration, "run"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let scope = registry
        .register_scope(
            &function,
            None,
            key(RustSourceNode::LexicalScope(1), "body"),
        )
        .unwrap();
    (registry, file, function, scope)
}

#[test]
fn copied_compiler_metadata_does_not_authenticate_cross_registry_references() {
    let (mut first, _, _, first_scope) = fixture(CFileRole::GeneratedSource);
    let (mut second, _, _, second_scope) = fixture(CFileRole::GeneratedSource);
    let local_key = key(RustSourceNode::Binding(2), "value");
    let ty = CObjectType::scalar(CScalarType::I32);
    let local = first
        .register_local(&first_scope, local_key.clone(), ty.clone())
        .unwrap();
    second.register_local(&second_scope, local_key, ty).unwrap();
    assert!(matches!(
        CExpressions::new(&second).local(local),
        Err(CExpressionError::Registry(CRegistryError::CrossRegistry))
    ));
}

#[test]
fn compiler_origins_cannot_claim_runtime_ownership() {
    for (role, accepted) in [
        (CFileRole::GeneratedSource, true),
        (CFileRole::RuntimeSource, false),
    ] {
        let (registry, file, function, scope) = fixture(role);
        let statements = CStatements::new(&registry, function.clone()).unwrap();
        let body = statements.block(scope, vec![]).unwrap();
        let declarations = CDeclarations::new(&registry, file).unwrap();
        let definition = declarations
            .function_definition(function, CLinkage::Internal, vec![], body)
            .unwrap();
        let source = declarations
            .source_file(vec![CFileItem::Definition(definition)])
            .unwrap();
        let result = registry.check_context(&[source]);
        if accepted {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CContextError::OriginRoleMismatch));
        }
    }
}

#[test]
fn compiler_binding_and_scope_nodes_keep_distinct_identity() {
    // References share metadata instead of embedding its strings at every node.
    assert!(std::mem::size_of::<CGeneratedOrigin>() <= 32);
    assert_ne!(
        key(RustSourceNode::Binding(3), "same"),
        key(RustSourceNode::LexicalScope(3), "same")
    );
}
