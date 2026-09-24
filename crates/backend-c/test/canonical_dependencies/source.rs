//! Minimal source-owned identity/relay fixture; imported owners may be unused.
use portable_backend_c::{ast::*, dialect::*};
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

pub fn source(
    root: RustDeclarationId,
    types: &[CDependencyStruct],
    calls: &[CDependencyFunction],
    record_parameter: bool,
) -> Result<CDependencyApi, String> {
    let stem = format!(
        "polyrust_owner_{}_{:x}",
        root.crate_id, root.definition_path_hash
    );
    source_named(root, types, calls, record_parameter, &stem)
}

pub fn source_named(
    root: RustDeclarationId,
    types: &[CDependencyStruct],
    calls: &[CDependencyFunction],
    record_parameter: bool,
    stem: &str,
) -> Result<CDependencyApi, String> {
    let mut r = CRegistry::new();
    for proof in types {
        r.import_struct(proof.clone()).map_err(|e| e.to_string())?;
    }
    let calls: Vec<_> = calls
        .iter()
        .map(|proof| r.import_function(proof.clone()))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let name = format!("{stem}_identity");
    let header = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("{stem}.h")).unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let implementation = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("{stem}.c")).unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let id = RustDeclarationId {
        crate_id: root.crate_id,
        definition_path_hash: root.definition_path_hash + 1,
    };
    let location = RustSourceLocation {
        file: format!("crate_{}/lib.rs", root.crate_id),
        line: 1,
        column: 0,
    };
    let ancestry = Arc::new(RustModuleDocumentation {
        declaration: root,
        parent: None,
        location: location.clone(),
        documentation: vec![],
    });
    let exports = Arc::new(RustCrateExports {
        root,
        modules: BTreeMap::from([(
            root,
            BTreeMap::from([(
                RustExportName {
                    namespace: RustExportNamespace::Value,
                    name: name.clone(),
                },
                RustExportTarget::Declaration(id),
            )]),
        )]),
        module_ancestries: BTreeMap::from([(root, vec![ancestry.clone()].into())]),
    });
    let ty = if record_parameter {
        CObjectType::structure(types[0].record().clone())
    } else {
        CObjectType::scalar(CScalarType::I32)
    };
    let signature = CFunctionType::new(
        CReturnType::Value(CReturnValue::new(ty.clone()).unwrap()),
        vec![CParameterType::new(ty).unwrap()],
    );
    let function = r
        .register_function(
            &header,
            CDeclarationKey {
                name: CIdentifier::new(&name).unwrap(),
                origin: CGeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                    declaration: id,
                    node: RustSourceNode::Declaration,
                    module: root,
                    location,
                    visibility: RustVisibility::Public,
                    externally_reachable: true,
                    documentation: vec![],
                    module_ancestors: vec![ancestry].into(),
                    crate_exports: exports,
                })),
            },
            signature,
        )
        .unwrap();
    let key = |name| CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
    };
    let scope = r.register_scope(&function, None, key("body")).unwrap();
    let parameter = r
        .register_parameter(&function, 0, key("input"), CConstness::Unqualified)
        .unwrap();
    let e = CExpressions::new(&r);
    let value = e.read(e.parameter(parameter.clone()).unwrap()).unwrap();
    let value = if let Some(call) = calls.first() {
        e.call_value(e.direct(call.clone()).unwrap(), vec![value])
            .unwrap()
    } else {
        value
    };
    let s = CStatements::new(&r, function.clone()).unwrap();
    let body = s
        .block(scope, vec![s.return_statement(Some(value)).unwrap()])
        .unwrap();
    let h = CDeclarations::new(&r, header).unwrap();
    let c = CDeclarations::new(&r, implementation).unwrap();
    let files = vec![
        h.source_file(vec![CFileItem::Declaration(
            h.function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
        )])
        .unwrap(),
        c.source_file(vec![CFileItem::Definition(
            c.function_definition(function, CLinkage::External, vec![parameter], body)
                .unwrap(),
        )])
        .unwrap(),
    ];
    let ast = project_c_package(r.freeze(), files).map_err(|e| format!("{e:?}"))?;
    let checked = verify_unresolved_package(&CDialect, ast).map_err(|e| format!("{e:?}"))?;
    let linked = TargetLinker::new(CDialect)
        .link_ast(&checked)
        .map_err(|e| format!("{e:?}"))?;
    let certificate = certify_resolved_package(&CDialect, linked).map_err(|e| format!("{e:?}"))?;
    CDependencyApi::from_certificate(certificate)
}

pub fn root(crate_id: u64) -> RustDeclarationId {
    RustDeclarationId {
        crate_id,
        definition_path_hash: 1000,
    }
}
