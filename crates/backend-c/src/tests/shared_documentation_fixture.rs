//! Small typed functions for documentation and certification mutation tests.
use crate::ast::*;
use portable_codegen::*;
use std::sync::Arc;

pub(super) fn metadata() -> RustSourceOrigin {
    let root = RustDeclarationId {
        crate_id: 1,
        definition_path_hash: 1,
    };
    let module = RustDeclarationId {
        definition_path_hash: 2,
        ..root
    };
    let location = RustSourceLocation {
        file: "source.rs".into(),
        line: 1,
        column: 0,
    };
    RustSourceOrigin {
        declaration: RustDeclarationId {
            definition_path_hash: 3,
            ..root
        },
        node: RustSourceNode::Declaration,
        module,
        location: location.clone(),
        visibility: RustVisibility::Public,
        externally_reachable: true,
        crate_exports: Arc::new(RustCrateExports {
            module_ancestries: std::collections::BTreeMap::new(),
            root,
            modules: Default::default(),
        }),
        documentation: vec!["function one".into(), "".into(), "function two */".into()],
        module_ancestors: vec![
            RustModuleDocumentation {
                declaration: root,
                parent: None,
                location: location.clone(),
                documentation: vec!["root".into()],
            },
            RustModuleDocumentation {
                declaration: module,
                parent: Some(root),
                location,
                documentation: vec!["child".into()],
            },
        ]
        .into_iter()
        .map(Arc::new)
        .collect::<Vec<_>>()
        .into(),
    }
}

pub(super) fn bare() -> RustSourceOrigin {
    let mut metadata = metadata();
    metadata.documentation.clear();
    for module in Arc::make_mut(&mut metadata.module_ancestors) {
        Arc::make_mut(module).documentation.clear();
    }
    metadata
}

pub(super) fn module(
    metadata: &mut RustSourceOrigin,
    index: usize,
) -> &mut RustModuleDocumentation {
    Arc::make_mut(&mut Arc::make_mut(&mut metadata.module_ancestors)[index])
}

pub(super) fn source(origins: Vec<RustSourceOrigin>) -> (CFrozenRegistry, CSourceFile) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("tests/documented.c").unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let mut prototypes = Vec::new();
    let mut definitions = Vec::new();
    for (index, origin) in origins.into_iter().enumerate() {
        let function = registry
            .register_function(
                &file,
                CDeclarationKey {
                    name: CIdentifier::new(&format!("document{index}")).unwrap(),
                    origin: CGeneratedOrigin::RustSource(std::sync::Arc::new(origin)),
                },
                CFunctionType::new(
                    CReturnType::Value(
                        CReturnValue::new(CObjectType::scalar(CScalarType::I32)).unwrap(),
                    ),
                    vec![],
                ),
            )
            .unwrap();
        let scope = registry
            .register_scope(
                &function,
                None,
                CDeclarationKey {
                    name: CIdentifier::new(&format!("body{index}")).unwrap(),
                    origin: CGeneratedOrigin::Synthesized(CSynthesisReason::EvaluationTemporary),
                },
            )
            .unwrap();
        let value = CExpressions::new(&registry)
            .literal(CLiteral::Signed(CSignedLiteral::I32(1)))
            .unwrap();
        let statements = CStatements::new(&registry, function.clone()).unwrap();
        let body = statements
            .block(
                scope,
                vec![statements.return_statement(Some(value)).unwrap()],
            )
            .unwrap();
        let declarations = CDeclarations::new(&registry, file.clone()).unwrap();
        prototypes.push(CFileItem::Declaration(
            declarations
                .function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
        ));
        definitions.push(CFileItem::Definition(
            declarations
                .function_definition(function, CLinkage::External, vec![], body)
                .unwrap(),
        ));
    }
    prototypes.extend(definitions);
    let source = CDeclarations::new(&registry, file)
        .unwrap()
        .source_file(prototypes)
        .unwrap();
    (registry.freeze(), source)
}
