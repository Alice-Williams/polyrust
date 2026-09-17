//! Typed target fixtures; these origins do not claim compiler admission.
use super::{CDependencyApi, CDependencyFunction, CDialect, project_c_package};
use crate::ast::*;
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
    }
}

pub(super) fn ast(
    crate_id: u64,
    imported: Option<CDependencyFunction>,
    scalar: bool,
) -> TargetAstPackage<CDialect> {
    let id = |hash| RustDeclarationId {
        crate_id,
        definition_path_hash: hash,
    };
    let location = RustSourceLocation {
        file: "src/lib.rs".into(),
        line: 1,
        column: 0,
    };
    let root = Arc::new(RustModuleDocumentation {
        declaration: id(1),
        parent: None,
        location: location.clone(),
        documentation: vec![],
    });
    let ancestry: RustModuleAncestry = vec![root].into();
    let exports = Arc::new(RustCrateExports {
        root: id(1),
        modules: BTreeMap::from([(
            id(1),
            BTreeMap::from([(
                RustExportName {
                    namespace: RustExportNamespace::Value,
                    name: "operation".into(),
                },
                RustExportTarget::Declaration(id(2)),
            )]),
        )]),
        module_ancestries: BTreeMap::from([(id(1), ancestry.clone())]),
    });
    let mut registry = CRegistry::new();
    let header = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("polyrust_void_{crate_id}.h")).unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let source = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("void_{crate_id}.c")).unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let imported = imported.map(|function| registry.import_function(function).unwrap());
    let result = if scalar {
        CReturnType::Value(CReturnValue::new(CObjectType::scalar(CScalarType::I32)).unwrap())
    } else {
        CReturnType::Void
    };
    let function = registry
        .register_function(
            &header,
            CDeclarationKey {
                name: CIdentifier::new(&format!("operation_{crate_id}")).unwrap(),
                origin: CGeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                    declaration: id(2),
                    node: RustSourceNode::Declaration,
                    module: id(1),
                    location,
                    visibility: RustVisibility::Public,
                    externally_reachable: true,
                    documentation: vec!["Ordinary void/result operation.".into()],
                    module_ancestors: ancestry,
                    crate_exports: exports,
                })),
            },
            CFunctionType::new(
                result,
                vec![CParameterType::new(CObjectType::scalar(CScalarType::I32)).unwrap()],
            ),
        )
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("body"))
        .unwrap();
    let parameter = registry
        .register_parameter(&function, 0, key("input"), CConstness::Unqualified)
        .unwrap();
    let expressions = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let input = expressions
        .read(expressions.parameter(parameter.clone()).unwrap())
        .unwrap();
    let mut body = vec![];
    if let Some(imported) = imported {
        let callable = expressions.direct(imported).unwrap();
        assert!(
            expressions
                .call_value(callable.clone(), vec![input.clone()])
                .is_err()
        );
        assert!(expressions.call_effect(callable.clone(), vec![]).is_err());
        assert!(
            expressions
                .call_effect(
                    callable.clone(),
                    vec![expressions.literal(CLiteral::Bool(true)).unwrap()]
                )
                .is_err()
        );
        let effect = expressions
            .call_effect(callable, vec![input.clone()])
            .unwrap();
        body.push(statements.evaluate(effect).unwrap());
    } else {
        body.push(statements.discard(input.clone()).unwrap());
    }
    if scalar {
        assert!(statements.return_statement(None).is_err());
        body.push(statements.return_statement(Some(input)).unwrap());
    } else {
        assert!(statements.return_statement(Some(input)).is_err());
        body.push(statements.return_statement(None).unwrap());
    }
    let body = statements.block(scope, body).unwrap();
    let declarations = CDeclarations::new(&registry, header).unwrap();
    let header = declarations
        .source_file(vec![CFileItem::Declaration(
            declarations
                .function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
        )])
        .unwrap();
    let definitions = CDeclarations::new(&registry, source).unwrap();
    let source = definitions
        .source_file(vec![CFileItem::Definition(
            definitions
                .function_definition(function, CLinkage::External, vec![parameter], body)
                .unwrap(),
        )])
        .unwrap();
    project_c_package(registry.freeze(), vec![header, source]).unwrap()
}

pub(super) fn package(
    crate_id: u64,
    imported: Option<CDependencyFunction>,
    scalar: bool,
) -> RenderReadyPackage<CDialect> {
    let checked = verify_unresolved_package(&CDialect, ast(crate_id, imported, scalar)).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    certify_resolved_package(&CDialect, linked).unwrap()
}

pub(super) fn chain() -> Vec<CDependencyApi> {
    let mut owners = vec![];
    for (id, scalar) in [(71, false), (72, false), (73, true)] {
        let imported = owners
            .last()
            .map(|owner: &CDependencyApi| owner.functions().next().unwrap().clone());
        owners.push(CDependencyApi::from_certificate(package(id, imported, scalar)).unwrap());
    }
    owners
}
