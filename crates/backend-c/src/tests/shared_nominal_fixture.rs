//! Independently owned packages using original certified result types.
use crate::{ast::*, dialect::*};
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone)]
pub(super) enum Operation {
    Copy,
    Construct,
    ScalarLocal,
    Payload,
    Tag,
    Call(CDependencyFunction),
}

pub(super) struct Fixture {
    pub registry: CFrozenRegistry,
    pub files: Vec<CSourceFile>,
    pub functions: Vec<CFunctionRef>,
}

pub(super) fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
    }
}

pub(super) fn origins(crate_id: u64, names: &[String]) -> Vec<RustSourceOrigin> {
    let id = |definition_path_hash| RustDeclarationId {
        crate_id,
        definition_path_hash,
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
            names
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    (
                        RustExportName {
                            namespace: RustExportNamespace::Value,
                            name: name.clone(),
                        },
                        RustExportTarget::Declaration(id(10 + index as u64)),
                    )
                })
                .collect(),
        )]),
        module_ancestries: BTreeMap::from([(id(1), vec![root.clone()].into())]),
    });
    names
        .iter()
        .enumerate()
        .map(|(index, _)| RustSourceOrigin {
            declaration: id(10 + index as u64),
            node: RustSourceNode::Declaration,
            module: id(1),
            location: location.clone(),
            visibility: RustVisibility::Public,
            externally_reachable: true,
            documentation: vec![],
            module_ancestors: vec![root.clone()].into(),
            crate_exports: exports.clone(),
        })
        .collect()
}

pub(super) fn fixture(
    crate_id: u64,
    proofs: &[CDependencyStruct],
    operations: &[Operation],
) -> Fixture {
    let mut r = CRegistry::new();
    let records: Vec<_> = proofs
        .iter()
        .map(|p| r.import_struct(p.clone()).unwrap())
        .collect();
    let ty = CObjectType::structure(records[0].clone());
    let i32_ty = CObjectType::scalar(CScalarType::I32);
    let bool_ty = CObjectType::scalar(CScalarType::Bool);
    let signatures: Vec<_> = operations
        .iter()
        .map(|operation| {
            let (result, parameters) = match operation {
                Operation::Copy => (ty.clone(), vec![ty.clone()]),
                Operation::Construct => (ty.clone(), vec![bool_ty.clone(), i32_ty.clone()]),
                Operation::ScalarLocal => (i32_ty.clone(), vec![bool_ty.clone(), i32_ty.clone()]),
                Operation::Payload => (i32_ty.clone(), vec![ty.clone()]),
                Operation::Tag => (bool_ty.clone(), vec![ty.clone()]),
                Operation::Call(proof) => return proof.signature().clone(),
            };
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(result).unwrap()),
                parameters
                    .into_iter()
                    .map(|ty| CParameterType::new(ty).unwrap())
                    .collect(),
            )
        })
        .collect();
    let calls: Vec<_> = operations
        .iter()
        .map(|operation| match operation {
            Operation::Call(proof) => Some(r.import_function(proof.clone()).unwrap()),
            _ => None,
        })
        .collect();
    let header = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("polyrust_nominal_{crate_id}.h")).unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let source = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("nominal_{crate_id}.c")).unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let names: Vec<_> = (0..operations.len())
        .map(|index| format!("nominal_{crate_id}_{index}"))
        .collect();
    let origins = origins(crate_id, &names);
    let functions: Vec<_> = signatures
        .into_iter()
        .enumerate()
        .map(|(index, signature)| {
            let mut key = key(&names[index]);
            key.origin = CGeneratedOrigin::RustSource(Arc::new(origins[index].clone()));
            r.register_function(&header, key, signature).unwrap()
        })
        .collect();
    let mut header_items = vec![];
    let mut items = vec![];
    for (index, function) in functions.iter().enumerate() {
        header_items.push(CFileItem::Declaration(
            CDeclarations::new(&r, header.clone())
                .unwrap()
                .function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
        ));
        let scope = r.register_scope(function, None, key("body")).unwrap();
        let parameters: Vec<_> = (0..function.signature().parameters().len())
            .map(|position| {
                r.register_parameter(
                    function,
                    position,
                    key(&format!("input{position}")),
                    CConstness::Unqualified,
                )
                .unwrap()
            })
            .collect();
        let local = matches!(
            operations[index],
            Operation::Construct | Operation::ScalarLocal
        )
        .then(|| r.register_local(&scope, key("value"), ty.clone()).unwrap());
        let e = CExpressions::new(&r);
        let s = CStatements::new(&r, function.clone()).unwrap();
        let read = |position: usize| {
            e.read(e.parameter(parameters[position].clone()).unwrap())
                .unwrap()
        };
        let mut statements = vec![];
        let value = match &operations[index] {
            Operation::Copy => read(0),
            Operation::Construct | Operation::ScalarLocal => {
                let fields = proofs[0]
                    .members()
                    .iter()
                    .enumerate()
                    .map(|(position, member)| {
                        (
                            member.clone(),
                            e.expression_initializer(read(position)).unwrap(),
                        )
                    })
                    .collect();
                let initial = e.struct_initializer(records[0].clone(), fields).unwrap();
                let local = local.unwrap();
                statements.push(s.declare(local.clone(), Some(initial)).unwrap());
                let place = e.local(local).unwrap();
                if matches!(operations[index], Operation::ScalarLocal) {
                    e.read(e.member(place, proofs[0].members()[1].clone()).unwrap())
                        .unwrap()
                } else {
                    e.read(place).unwrap()
                }
            }
            Operation::Payload | Operation::Tag => {
                let position = usize::from(matches!(operations[index], Operation::Payload));
                e.read(
                    e.member(
                        e.parameter(parameters[0].clone()).unwrap(),
                        proofs[0].members()[position].clone(),
                    )
                    .unwrap(),
                )
                .unwrap()
            }
            Operation::Call(_) => e
                .call_value(
                    e.direct(calls[index].clone().unwrap()).unwrap(),
                    (0..parameters.len()).map(read).collect(),
                )
                .unwrap(),
        };
        statements.push(s.return_statement(Some(value)).unwrap());
        let body = s.block(scope, statements).unwrap();
        items.push(CFileItem::Definition(
            CDeclarations::new(&r, source.clone())
                .unwrap()
                .function_definition(function.clone(), CLinkage::External, parameters, body)
                .unwrap(),
        ));
    }
    let files = vec![
        CDeclarations::new(&r, header)
            .unwrap()
            .source_file(header_items)
            .unwrap(),
        CDeclarations::new(&r, source)
            .unwrap()
            .source_file(items)
            .unwrap(),
    ];
    Fixture {
        registry: r.freeze(),
        files,
        functions,
    }
}

pub(super) fn certify(fixture: Fixture) -> Result<RenderReadyPackage<CDialect>, String> {
    let ast = project_c_package(fixture.registry, fixture.files).map_err(|e| format!("{e:?}"))?;
    let checked = verify_unresolved_package(&CDialect, ast).map_err(|e| format!("{e:?}"))?;
    let linked = TargetLinker::new(CDialect)
        .link_ast(&checked)
        .map_err(|e| format!("{e:?}"))?;
    certify_resolved_package(&CDialect, linked).map_err(|e| format!("{e:?}"))
}

pub(super) fn producer() -> CDependencyApi {
    let (registry, files) = super::result_fixture::public_fixture(
        super::result_fixture::Mutation::None,
        super::result_fixture::PublicApi::ResultSignatures,
    );
    CDependencyApi::from_certificate(
        certify(Fixture {
            registry,
            files,
            functions: vec![],
        })
        .unwrap(),
    )
    .unwrap()
}
