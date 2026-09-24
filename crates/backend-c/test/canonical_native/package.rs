//! Executable test packages consume original canonical witnesses, never copied tags.
use super::origins::{key, origins};
use portable_backend_c::{ast::*, dialect::*};
use portable_codegen::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub enum Fault {
    None,
    SwapTag,
    ZeroPayload,
}

pub enum Operation {
    Construct(Fault),
    Forward,
    Tag,
    Payload,
    Compose {
        inner: Box<CDependencyFunction>,
        outer: Box<CDependencyFunction>,
        duplicate: bool,
    },
}

impl Operation {
    fn name(&self) -> &'static str {
        match self {
            Self::Construct(_) => "construct",
            Self::Forward => "forward",
            Self::Tag => "tag",
            Self::Payload => "payload",
            Self::Compose { .. } => "compose",
        }
    }
    fn signature(&self, record: &CObjectType) -> CFunctionType {
        let (result, parameters) = match self {
            Self::Construct(_) | Self::Compose { .. } => (
                record.clone(),
                vec![
                    CObjectType::scalar(CScalarType::Bool),
                    CObjectType::scalar(CScalarType::I32),
                ],
            ),
            Self::Forward => (record.clone(), vec![record.clone()]),
            Self::Tag => (CObjectType::scalar(CScalarType::Bool), vec![record.clone()]),
            Self::Payload => (CObjectType::scalar(CScalarType::I32), vec![record.clone()]),
        };
        CFunctionType::new(
            CReturnType::Value(CReturnValue::new(result).unwrap()),
            parameters
                .into_iter()
                .map(|ty| CParameterType::new(ty).unwrap())
                .collect(),
        )
    }
}

pub fn function(api: &CDependencyApi, suffix: &str) -> CDependencyFunction {
    api.functions()
        .find(|f| f.symbol().as_str().ends_with(&format!("_{suffix}")))
        .unwrap()
        .clone()
}

pub fn package(
    crate_id: u64,
    proof: &CDependencyStruct,
    extra: &[CDependencyStruct],
    operations: &[Operation],
    reverse_imports: bool,
) -> CDependencyApi {
    let mut r = CRegistry::new();
    let mut types: Vec<_> = std::iter::once(proof).chain(extra).collect();
    if reverse_imports {
        types.reverse();
    }
    for ty in types {
        r.import_struct(ty.clone()).unwrap();
    }
    let record = CObjectType::structure(proof.record().clone());
    let imports: Vec<_> = operations
        .iter()
        .map(|op| match op {
            Operation::Compose { inner, outer, .. } => {
                let (inner_ref, outer_ref) = if reverse_imports {
                    let b = r.import_function(outer.as_ref().clone()).unwrap();
                    (r.import_function(inner.as_ref().clone()).unwrap(), b)
                } else {
                    let a = r.import_function(inner.as_ref().clone()).unwrap();
                    (a, r.import_function(outer.as_ref().clone()).unwrap())
                };
                Some((inner_ref, outer_ref))
            }
            _ => None,
        })
        .collect();
    let stem = format!("polyrust_canonical_{crate_id}");
    let file = |extension, role| CFileKey {
        path: RelativeOutputPath::new(format!("{stem}.{extension}")).unwrap(),
        role,
    };
    let header = r
        .register_file(file("h", CFileRole::GeneratedPublicHeader))
        .unwrap();
    let implementation = r
        .register_file(file("c", CFileRole::GeneratedSource))
        .unwrap();
    let names: Vec<_> = operations
        .iter()
        .map(|op| format!("{stem}_{}", op.name()))
        .collect();
    let origins = origins(crate_id, &names);
    let functions: Vec<_> = operations
        .iter()
        .enumerate()
        .map(|(i, op)| {
            r.register_function(
                &header,
                CDeclarationKey {
                    name: CIdentifier::new(&names[i]).unwrap(),
                    origin: CGeneratedOrigin::RustSource(Arc::new(origins[i].clone())),
                },
                op.signature(&record),
            )
            .unwrap()
        })
        .collect();
    let mut headers = vec![];
    let mut definitions = vec![];
    for (i, function) in functions.iter().enumerate() {
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
            operations[i],
            Operation::Construct(_) | Operation::Compose { .. }
        )
        .then(|| {
            r.register_local(&scope, key("result"), record.clone())
                .unwrap()
        });
        let e = CExpressions::new(&r);
        let s = CStatements::new(&r, function.clone()).unwrap();
        let read = |position: usize| {
            e.read(e.parameter(parameters[position].clone()).unwrap())
                .unwrap()
        };
        let mut statements = vec![];
        let value = match &operations[i] {
            Operation::Construct(fault) => {
                let tag = if matches!(fault, Fault::SwapTag) {
                    e.numeric_conversion(
                        CScalarType::Bool,
                        e.unary(CUnaryOperator::LogicalNot, read(0)).unwrap(),
                    )
                    .unwrap()
                } else {
                    read(0)
                };
                let payload = if matches!(fault, Fault::ZeroPayload) {
                    statements.push(s.discard(read(1)).unwrap());
                    e.literal(CLiteral::Signed(CSignedLiteral::I32(0))).unwrap()
                } else {
                    read(1)
                };
                let fields = proof
                    .members()
                    .iter()
                    .cloned()
                    .zip([tag, payload])
                    .map(|(member, value)| (member, e.expression_initializer(value).unwrap()))
                    .collect();
                let initializer = e
                    .struct_initializer(proof.record().clone(), fields)
                    .unwrap();
                let local = local.unwrap();
                statements.push(s.declare(local.clone(), Some(initializer)).unwrap());
                e.read(e.local(local).unwrap()).unwrap()
            }
            Operation::Forward => read(0),
            Operation::Tag | Operation::Payload => {
                let field = usize::from(matches!(operations[i], Operation::Payload));
                e.read(
                    e.member(
                        e.parameter(parameters[0].clone()).unwrap(),
                        proof.members()[field].clone(),
                    )
                    .unwrap(),
                )
                .unwrap()
            }
            Operation::Compose { duplicate, .. } => {
                let (inner, outer) = imports[i].as_ref().unwrap();
                let call = || {
                    e.call_value(e.direct(inner.clone()).unwrap(), vec![read(0), read(1)])
                        .unwrap()
                };
                if *duplicate {
                    statements.push(s.discard(call()).unwrap());
                }
                let local = local.unwrap();
                statements.push(
                    s.declare(
                        local.clone(),
                        Some(e.expression_initializer(call()).unwrap()),
                    )
                    .unwrap(),
                );
                let argument = e.read(e.local(local).unwrap()).unwrap();
                e.call_value(e.direct(outer.clone()).unwrap(), vec![argument])
                    .unwrap()
            }
        };
        statements.push(s.return_statement(Some(value)).unwrap());
        let body = s.block(scope, statements).unwrap();
        headers.push(CFileItem::Declaration(
            CDeclarations::new(&r, header.clone())
                .unwrap()
                .function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
        ));
        definitions.push(CFileItem::Definition(
            CDeclarations::new(&r, implementation.clone())
                .unwrap()
                .function_definition(function.clone(), CLinkage::External, parameters, body)
                .unwrap(),
        ));
    }
    let files = vec![
        CDeclarations::new(&r, header)
            .unwrap()
            .source_file(headers)
            .unwrap(),
        CDeclarations::new(&r, implementation)
            .unwrap()
            .source_file(definitions)
            .unwrap(),
    ];
    let ast = project_c_package(r.freeze(), files).unwrap();
    let checked = verify_unresolved_package(&CDialect, ast).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    CDependencyApi::from_certificate(certify_resolved_package(&CDialect, linked).unwrap()).unwrap()
}

pub fn producer(crate_id: u64, proof: &CDependencyStruct, fault: Fault) -> CDependencyApi {
    package(
        crate_id,
        proof,
        &[],
        &[
            Operation::Construct(fault),
            Operation::Forward,
            Operation::Tag,
            Operation::Payload,
        ],
        false,
    )
}
