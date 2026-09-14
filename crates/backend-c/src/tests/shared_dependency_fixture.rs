//! Small scalar crates with independently owned files and public APIs.
use crate::{ast::*, dialect::*};
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

pub(super) struct Fixture {
    pub registry: CFrozenRegistry,
    pub files: Vec<CSourceFile>,
    pub functions: Vec<CFunctionRef>,
    pub imported: Vec<CFunctionRef>,
}

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
    }
}

pub(super) fn fixture(
    crate_id: u64,
    scalars: &[CScalarType],
    dependencies: &[CDependencyFunction],
    calls: &[Option<usize>],
) -> Fixture {
    configured(
        PackageIdentity {
            crate_id,
            file_stem: format!("polyrust_dep_{crate_id}"),
            definition_base: 10,
        },
        scalars,
        dependencies,
        calls,
    )
}

pub(super) struct PackageIdentity {
    pub crate_id: u64,
    pub file_stem: String,
    pub definition_base: u64,
}

pub(super) fn configured(
    identity: PackageIdentity,
    scalars: &[CScalarType],
    dependencies: &[CDependencyFunction],
    calls: &[Option<usize>],
) -> Fixture {
    build(identity, scalars, dependencies, calls, None)
}

pub(super) fn named(
    crate_id: u64,
    names: &[&str],
    dependencies: &[CDependencyFunction],
    calls: &[Option<usize>],
) -> Fixture {
    build(
        PackageIdentity {
            crate_id,
            file_stem: format!("polyrust_dep_{crate_id}"),
            definition_base: 10,
        },
        &vec![CScalarType::I32; names.len()],
        dependencies,
        calls,
        Some(names),
    )
}

fn build(
    identity: PackageIdentity,
    scalars: &[CScalarType],
    dependencies: &[CDependencyFunction],
    calls: &[Option<usize>],
    names: Option<&[&str]>,
) -> Fixture {
    let crate_id = identity.crate_id;
    assert_eq!(scalars.len(), calls.len());
    let id = |hash| RustDeclarationId {
        crate_id,
        definition_path_hash: hash,
    };
    let location = RustSourceLocation {
        file: format!("crate_{crate_id}/src/lib.rs"),
        line: 1,
        column: 0,
    };
    let root = Arc::new(RustModuleDocumentation {
        declaration: id(1),
        parent: None,
        location: location.clone(),
        documentation: vec![],
    });
    let exports = Arc::new(RustCrateExports {
        root: id(1),
        modules: BTreeMap::from([(
            id(1),
            scalars
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    (
                        RustExportName {
                            namespace: RustExportNamespace::Value,
                            name: format!("api{index}"),
                        },
                        RustExportTarget::Declaration(id(identity.definition_base + index as u64)),
                    )
                })
                .collect(),
        )]),
        module_ancestries: BTreeMap::from([(id(1), vec![root.clone()].into())]),
    });
    let mut registry = CRegistry::new();
    let imported: Vec<_> = dependencies
        .iter()
        .map(|dependency| registry.import_function(dependency.clone()).unwrap())
        .collect();
    let header = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("{}.h", identity.file_stem)).unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let implementation = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("{}.c", identity.file_stem)).unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let mut functions = Vec::new();
    let mut parameters = Vec::new();
    let mut scopes = Vec::new();
    for (index, scalar) in scalars.iter().enumerate() {
        let ty = CObjectType::scalar(*scalar);
        let function = registry
            .register_function(
                &header,
                CDeclarationKey {
                    name: CIdentifier::new(names.map(|names| names[index]).unwrap_or(&format!(
                        "poly_fn_{crate_id:016x}_{:016x}",
                        identity.definition_base + index as u64
                    )))
                    .unwrap(),
                    origin: CGeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                        declaration: id(identity.definition_base + index as u64),
                        node: RustSourceNode::Declaration,
                        module: id(1),
                        location: location.clone(),
                        visibility: RustVisibility::Public,
                        externally_reachable: true,
                        documentation: vec![],
                        module_ancestors: vec![root.clone()].into(),
                        crate_exports: exports.clone(),
                    })),
                },
                CFunctionType::new(
                    CReturnType::Value(CReturnValue::new(ty.clone()).unwrap()),
                    vec![CParameterType::new(ty).unwrap()],
                ),
            )
            .unwrap();
        parameters.push(
            registry
                .register_parameter(&function, 0, key("input"), CConstness::Unqualified)
                .unwrap(),
        );
        scopes.push(
            registry
                .register_scope(&function, None, key("body"))
                .unwrap(),
        );
        functions.push(function);
    }
    let expressions = CExpressions::new(&registry);
    let declarations = CDeclarations::new(&registry, header).unwrap();
    let header = declarations
        .source_file(
            functions
                .iter()
                .map(|function| {
                    CFileItem::Declaration(
                        declarations
                            .function_prototype(function.clone(), CLinkage::External)
                            .unwrap(),
                    )
                })
                .collect(),
        )
        .unwrap();
    let declarations = CDeclarations::new(&registry, implementation).unwrap();
    let source = declarations
        .source_file(
            functions
                .iter()
                .enumerate()
                .map(|(index, function)| {
                    let statements = CStatements::new(&registry, function.clone()).unwrap();
                    let input = expressions
                        .read(expressions.parameter(parameters[index].clone()).unwrap())
                        .unwrap();
                    let value = if let Some(callee) = calls[index] {
                        expressions
                            .call_value(
                                expressions.direct(imported[callee].clone()).unwrap(),
                                vec![input],
                            )
                            .unwrap()
                    } else {
                        input
                    };
                    let body = statements
                        .block(
                            scopes[index].clone(),
                            vec![statements.return_statement(Some(value)).unwrap()],
                        )
                        .unwrap();
                    CFileItem::Definition(
                        declarations
                            .function_definition(
                                function.clone(),
                                CLinkage::External,
                                vec![parameters[index].clone()],
                                body,
                            )
                            .unwrap(),
                    )
                })
                .collect(),
        )
        .unwrap();
    Fixture {
        registry: registry.freeze(),
        files: vec![header, source],
        functions,
        imported,
    }
}

pub(super) fn linked(fixture: &Fixture) -> LinkedTargetPackage<CDialect> {
    let ast = project_c_package(fixture.registry.clone(), fixture.files.clone()).unwrap();
    let verified = verify_unresolved_package(&CDialect, ast).unwrap();
    TargetLinker::new(CDialect).link_ast(&verified).unwrap()
}

pub(super) fn api(crate_id: u64, scalars: &[CScalarType]) -> CDependencyApi {
    let fixture = fixture(crate_id, scalars, &[], &vec![None; scalars.len()]);
    let certificate = certify_resolved_package(&CDialect, linked(&fixture)).unwrap();
    CDependencyApi::from_certificate(certificate).unwrap()
}
