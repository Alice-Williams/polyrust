//! Assertions inspect the actual typed tree created from compiler HIR.
use portable_backend_c::ast::{
    CAggregateRef, CDeclarationKey, CDeclarationKind, CDefinitionKind, CFileItem, CGeneratedOrigin,
    CInitializerKind, CSourceFile, CStatementKind,
};
use portable_codegen::{RustSourceNode, RustSourceOrigin, RustVisibility};

pub(super) fn dependencies(
    registry: &portable_backend_c::ast::CFrozenRegistry,
    file: &CSourceFile,
) {
    use portable_backend_c::dialect::{CHeader, CTypeRequirement, file_dependencies};
    let files = std::slice::from_ref(file);
    let result = file_dependencies(registry, files).expect("compiler tree dependencies");
    assert_eq!(result.len(), 1);
    let dependencies = &result[0];
    assert_eq!(dependencies.file(), file.identity());
    assert_eq!(
        dependencies.headers(),
        &std::collections::BTreeSet::from([CHeader::Stdint])
    );
    assert!(dependencies.libraries().is_empty());
    assert!(dependencies.aliases().is_empty());
    assert!(dependencies.functions().is_empty());
    assert!(dependencies.objects().is_empty());
    assert_eq!(dependencies.tags().len(), 2);
    assert!(
        dependencies
            .tags()
            .values()
            .all(|need| *need == CTypeRequirement::Complete)
    );
    // Repeated discovery cannot accumulate includes or depend on mutable state.
    assert_eq!(result, file_dependencies(registry, files).unwrap());
    assert_eq!(result, file_dependencies(registry, files).unwrap());
}

fn origin(key: &CDeclarationKey) -> &RustSourceOrigin {
    let CGeneratedOrigin::RustSource(origin) = &key.origin else {
        panic!("source declaration lost compiler provenance");
    };
    origin
}

fn source(key: &CDeclarationKey, line: usize, doc: &str) {
    let origin = origin(key);
    assert_eq!(origin.node, RustSourceNode::Declaration);
    assert_eq!(origin.location.line, line);
    assert!(origin.location.file.ends_with("origin_probe.rs"));
    assert_eq!(origin.documentation.len(), 1);
    assert_eq!(origin.documentation[0].trim(), doc);
}

pub(super) fn check(file: &CSourceFile) {
    let aggregates: Vec<_> = file
        .items()
        .iter()
        .filter_map(|item| match item {
            CFileItem::Declaration(declaration) => match declaration.kind() {
                CDeclarationKind::Aggregate { owner, members } => Some((owner, members)),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(aggregates.len(), 2);
    let (hidden, hidden_fields) = aggregates[0];
    let (public, public_fields) = aggregates[1];
    source(hidden.key(), 3, "Hidden ticket docs.");
    source(public.key(), 9, "Public ticket docs.");
    let hidden_origin = origin(hidden.key());
    let public_origin = origin(public.key());
    assert_ne!(hidden_origin.declaration, public_origin.declaration);
    assert_ne!(hidden_origin.module, public_origin.module);
    assert_eq!(
        hidden_origin.declaration.crate_id,
        public_origin.declaration.crate_id
    );
    assert_eq!(hidden_origin.visibility, RustVisibility::Public);
    assert_eq!(public_origin.visibility, RustVisibility::Public);
    assert!(!hidden_origin.externally_reachable);
    assert!(public_origin.externally_reachable);
    assert_eq!(hidden_fields.len(), 1);
    assert_eq!(public_fields.len(), 1);
    assert_eq!(
        origin(hidden_fields[0].key()).visibility,
        RustVisibility::Public
    );
    assert_eq!(
        origin(public_fields[0].key()).visibility,
        RustVisibility::RestrictedTo(public_origin.module)
    );

    let functions: Vec<_> = file
        .items()
        .iter()
        .filter_map(|item| match item {
            CFileItem::Definition(definition) => match definition.kind() {
                CDefinitionKind::Function {
                    function,
                    parameters,
                    body,
                    ..
                } => Some((function, parameters, body)),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(functions.len(), 1);
    let (function, parameters, body) = functions[0];
    source(function.key(), 14, "Score docs.");
    let function_origin = origin(function.key());
    assert_ne!(function_origin.declaration, public_origin.declaration);
    assert_eq!(function_origin.module, public_origin.module);
    assert!(function_origin.externally_reachable);
    assert_eq!(parameters.len(), 1);
    assert!(matches!(
        origin(parameters[0].key()).node,
        RustSourceNode::Parameter(_)
    ));
    assert!(matches!(
        origin(body.scope().key()).node,
        RustSourceNode::LexicalScope(_)
    ));
    assert_eq!(
        origin(body.scope().key()).declaration,
        function_origin.declaration
    );

    // Same source spelling in two modules must remain two actual C nominal owners.
    let locals: Vec<_> = body
        .statements()
        .iter()
        .filter_map(|statement| match statement.kind() {
            CStatementKind::Declare(local) => Some(local),
            _ => None,
        })
        .collect();
    assert_eq!(locals.len(), 4);
    assert!(matches!(
        body.statements()[0].kind(),
        CStatementKind::Discard(_)
    ));
    for (local, expected_owner) in locals.into_iter().zip([hidden, public]) {
        assert!(matches!(
            origin(local.local().key()).node,
            RustSourceNode::Binding(_)
        ));
        let CInitializerKind::Struct { owner, members } = local.initializer().unwrap().kind()
        else {
            panic!("expected existing C aggregate initializer");
        };
        assert_eq!(&CAggregateRef::Struct(owner.clone()), expected_owner);
        assert_eq!(members[0].0.owner(), expected_owner);
    }
}
