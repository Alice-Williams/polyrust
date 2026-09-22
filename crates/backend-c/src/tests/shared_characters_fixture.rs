//! Small source-owned target packages; Unicode validity is not a U32 type claim.
use crate::dialect::shared::dependency_fixture as f;
use crate::{ast::*, dialect::CDependencyFunction};
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

pub(super) const BOUNDARIES: [char; 19] = [
    '\u{0}',
    '\u{1}',
    '\u{7f}',
    '\u{80}',
    '\u{ff}',
    '\u{100}',
    '\u{378}',
    '\u{7ff}',
    '\u{800}',
    '\u{d7ff}',
    '\u{e000}',
    '\u{fdd0}',
    '\u{fffe}',
    '\u{ffff}',
    '\u{10000}',
    '\u{1f980}',
    '\u{f0000}',
    '\u{10fffe}',
    '\u{10ffff}',
];
pub(super) const OPERATORS: [CBinaryOperator; 6] = [
    CBinaryOperator::Equal,
    CBinaryOperator::NotEqual,
    CBinaryOperator::Less,
    CBinaryOperator::LessEqual,
    CBinaryOperator::Greater,
    CBinaryOperator::GreaterEqual,
];

#[derive(Clone)]
enum Body {
    Literal(char),
    Identity,
    Local,
    Select,
    Compare(CBinaryOperator),
    Forward,
}
impl Body {
    fn parameters(&self) -> Vec<CScalarType> {
        match self {
            Self::Literal(_) => vec![],
            Self::Identity | Self::Local | Self::Forward => vec![CScalarType::U32],
            Self::Select => vec![CScalarType::Bool, CScalarType::U32, CScalarType::U32],
            Self::Compare(_) => vec![CScalarType::U32; 2],
        }
    }
    fn result(&self) -> CScalarType {
        if matches!(self, Self::Compare(_)) {
            CScalarType::Bool
        } else {
            CScalarType::U32
        }
    }
}
fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
    }
}
pub(super) fn build(crate_id: u64, dependency: Option<CDependencyFunction>) -> f::Fixture {
    let mut specs: Vec<(String, Body)> = if dependency.is_some() {
        vec![("forward".into(), Body::Forward)]
    } else {
        BOUNDARIES
            .iter()
            .enumerate()
            .map(|(i, c)| (format!("literal_{i}"), Body::Literal(*c)))
            .collect()
    };
    if dependency.is_none() {
        specs.extend([
            ("identity".into(), Body::Identity),
            ("local".into(), Body::Local),
            ("select".into(), Body::Select),
        ]);
        specs.extend(
            OPERATORS
                .into_iter()
                .enumerate()
                .map(|(i, op)| (format!("compare_{i}"), Body::Compare(op))),
        );
    }
    let id = |hash| RustDeclarationId {
        crate_id,
        definition_path_hash: hash,
    };
    let location = RustSourceLocation {
        file: format!("crate_{crate_id}/lib.rs"),
        line: 1,
        column: 0,
    };
    let module = Arc::new(RustModuleDocumentation {
        declaration: id(1),
        parent: None,
        location: location.clone(),
        documentation: vec![],
    });
    let exports = Arc::new(RustCrateExports {
        root: id(1),
        modules: BTreeMap::from([(
            id(1),
            specs
                .iter()
                .enumerate()
                .map(|(i, (name, _))| {
                    (
                        RustExportName {
                            namespace: RustExportNamespace::Value,
                            name: name.clone(),
                        },
                        RustExportTarget::Declaration(id(10 + i as u64)),
                    )
                })
                .collect(),
        )]),
        module_ancestries: BTreeMap::from([(id(1), vec![module.clone()].into())]),
    });
    let mut registry = CRegistry::new();
    let imported = dependency
        .map(|value| registry.import_function(value).unwrap())
        .into_iter()
        .collect::<Vec<_>>();
    let mut file = |extension, role| {
        registry
            .register_file(CFileKey {
                path: RelativeOutputPath::new(format!(
                    "polyrust_characters_{crate_id}.{extension}"
                ))
                .unwrap(),
                role,
            })
            .unwrap()
    };
    let header = file("h", CFileRole::GeneratedPublicHeader);
    let implementation = file("c", CFileRole::GeneratedSource);
    let mut functions = Vec::new();
    let mut parameters = Vec::new();
    let mut scopes = Vec::new();
    let mut locals = Vec::new();
    for (index, (name, mode)) in specs.iter().enumerate() {
        let function = registry
            .register_function(
                &header,
                CDeclarationKey {
                    name: CIdentifier::new(&format!("poly_char_{crate_id}_{name}")).unwrap(),
                    origin: CGeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                        declaration: id(10 + index as u64),
                        node: RustSourceNode::Declaration,
                        module: id(1),
                        location: location.clone(),
                        visibility: RustVisibility::Public,
                        externally_reachable: true,
                        documentation: vec![],
                        module_ancestors: vec![module.clone()].into(),
                        crate_exports: exports.clone(),
                    })),
                },
                CFunctionType::new(
                    CReturnType::Value(
                        CReturnValue::new(CObjectType::scalar(mode.result())).unwrap(),
                    ),
                    mode.parameters()
                        .into_iter()
                        .map(|s| CParameterType::new(CObjectType::scalar(s)).unwrap())
                        .collect(),
                ),
            )
            .unwrap();
        let scope = registry
            .register_scope(&function, None, key("body"))
            .unwrap();
        let params = mode
            .parameters()
            .iter()
            .enumerate()
            .map(|(i, _)| {
                registry
                    .register_parameter(
                        &function,
                        i,
                        key(&format!("input_{i}")),
                        CConstness::Unqualified,
                    )
                    .unwrap()
            })
            .collect::<Vec<_>>();
        locals.push(matches!(mode, Body::Local | Body::Forward).then(|| {
            registry
                .register_local(
                    &scope,
                    key("value"),
                    CObjectType::scalar(CScalarType::U32)
                        .with_constness(CConstness::Const)
                        .unwrap(),
                )
                .unwrap()
        }));
        functions.push(function);
        parameters.push(params);
        scopes.push(scope);
    }
    let e = CExpressions::new(&registry);
    let d = CDeclarations::new(&registry, header).unwrap();
    let header = d
        .source_file(
            functions
                .iter()
                .map(|function| {
                    CFileItem::Declaration(
                        d.function_prototype(function.clone(), CLinkage::External)
                            .unwrap(),
                    )
                })
                .collect(),
        )
        .unwrap();
    let d = CDeclarations::new(&registry, implementation).unwrap();
    let source = d
        .source_file(
            specs
                .iter()
                .enumerate()
                .map(|(index, (_, mode))| {
                    let function = &functions[index];
                    let s = CStatements::new(&registry, function.clone()).unwrap();
                    let input = |i: usize| {
                        e.read(e.parameter(parameters[index][i].clone()).unwrap())
                            .unwrap()
                    };
                    let mut prefix = vec![];
                    let value = match mode {
                        Body::Literal(value) => e
                            .literal(CLiteral::Unsigned(CUnsignedLiteral::U32(u32::from(*value))))
                            .unwrap(),
                        Body::Identity => input(0),
                        Body::Local | Body::Forward => {
                            let value = if matches!(mode, Body::Forward) {
                                e.call_value(e.direct(imported[0].clone()).unwrap(), vec![input(0)])
                                    .unwrap()
                            } else {
                                input(0)
                            };
                            let local = locals[index].clone().unwrap();
                            prefix.push(
                                s.declare(
                                    local.clone(),
                                    Some(e.expression_initializer(value).unwrap()),
                                )
                                .unwrap(),
                            );
                            e.read(e.local(local).unwrap()).unwrap()
                        }
                        Body::Select => e.conditional(input(0), input(1), input(2)).unwrap(),
                        Body::Compare(op) => e
                            .numeric_conversion(
                                CScalarType::Bool,
                                e.binary(*op, input(0), input(1)).unwrap(),
                            )
                            .unwrap(),
                    };
                    prefix.push(s.return_statement(Some(value)).unwrap());
                    CFileItem::Definition(
                        d.function_definition(
                            function.clone(),
                            CLinkage::External,
                            parameters[index].clone(),
                            s.block(scopes[index].clone(), prefix).unwrap(),
                        )
                        .unwrap(),
                    )
                })
                .collect(),
        )
        .unwrap();
    f::Fixture {
        registry: registry.freeze(),
        files: vec![header, source],
        functions,
        imported,
    }
}
