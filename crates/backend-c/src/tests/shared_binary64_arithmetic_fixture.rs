//! Two-operand packages with original imports and explicit full-expression order.
use super::*;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy)]
pub(super) enum Body {
    Arithmetic,
    Forward,
}
fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
    }
}
pub(super) fn build(
    crate_id: u64,
    dependencies: &[crate::dialect::CDependencyFunction],
    mode: Body,
) -> f::Fixture {
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
            (0..6)
                .map(|index| {
                    (
                        RustExportName {
                            namespace: RustExportNamespace::Value,
                            name: format!("api{index}"),
                        },
                        RustExportTarget::Declaration(id(10 + index)),
                    )
                })
                .collect(),
        )]),
        module_ancestries: BTreeMap::from([(id(1), vec![module.clone()].into())]),
    });
    let mut registry = CRegistry::new();
    let imported: Vec<_> = dependencies
        .iter()
        .map(|dependency| registry.import_function(dependency.clone()).unwrap())
        .collect();
    let header = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("polyrust_arithmetic_{crate_id}.h")).unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let implementation = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("polyrust_arithmetic_{crate_id}.c")).unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let mut functions = Vec::new();
    let mut parameters = Vec::new();
    let mut scopes = Vec::new();
    let mut locals = Vec::new();
    for index in 0..6 {
        let ty = CObjectType::scalar(CScalarType::F64);
        let function = registry
            .register_function(
                &header,
                CDeclarationKey {
                    name: CIdentifier::new(&format!("poly_arithmetic_{crate_id}_{index}")).unwrap(),
                    origin: CGeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                        declaration: id(10 + index),
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
                    CReturnType::Value(CReturnValue::new(ty.clone()).unwrap()),
                    vec![CParameterType::new(ty.clone()).unwrap(); 2],
                ),
            )
            .unwrap();
        parameters.push([
            registry
                .register_parameter(&function, 0, key("left"), CConstness::Unqualified)
                .unwrap(),
            registry
                .register_parameter(&function, 1, key("right"), CConstness::Unqualified)
                .unwrap(),
        ]);
        let scope = registry
            .register_scope(&function, None, key("body"))
            .unwrap();
        locals.push(matches!(mode, Body::Arithmetic).then(|| {
            [
                registry
                    .register_local(&scope, key("left_value"), ty.clone())
                    .unwrap(),
                registry
                    .register_local(&scope, key("right_value"), ty.clone())
                    .unwrap(),
            ]
        }));
        scopes.push(scope);
        functions.push(function);
    }
    let e = CExpressions::new(&registry);
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
                    let s = CStatements::new(&registry, function.clone()).unwrap();
                    let inputs = parameters[index]
                        .iter()
                        .map(|parameter| e.read(e.parameter(parameter.clone()).unwrap()).unwrap())
                        .collect::<Vec<_>>();
                    let (statements, value) = match mode {
                        Body::Forward => (
                            vec![],
                            e.call_value(e.direct(imported[index].clone()).unwrap(), inputs)
                                .unwrap(),
                        ),
                        Body::Arithmetic => {
                            assert_eq!(imported.len(), 2);
                            let locals = locals[index].as_ref().unwrap();
                            let statements = (0..2)
                                .map(|side| {
                                    let call = e
                                        .call_value(
                                            e.direct(imported[side].clone()).unwrap(),
                                            vec![inputs[side].clone()],
                                        )
                                        .unwrap();
                                    s.declare(
                                        locals[side].clone(),
                                        Some(e.expression_initializer(call).unwrap()),
                                    )
                                    .unwrap()
                                })
                                .collect::<Vec<_>>();
                            let left = e.read(e.local(locals[0].clone()).unwrap()).unwrap();
                            let right = e.read(e.local(locals[1].clone()).unwrap()).unwrap();
                            let value = expression(&e, index, left, right);
                            (statements, value)
                        }
                    };
                    let mut statements = statements;
                    statements.push(s.return_statement(Some(value)).unwrap());
                    let body = s.block(scopes[index].clone(), statements).unwrap();
                    CFileItem::Definition(
                        declarations
                            .function_definition(
                                function.clone(),
                                CLinkage::External,
                                parameters[index].to_vec(),
                                body,
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
fn expression(e: &CExpressions<'_>, index: usize, left: CValue, right: CValue) -> CValue {
    match index {
        0..=3 => e.binary(OPERATORS[index], left, right).unwrap(),
        4 => e
            .binary(
                CBinaryOperator::Multiply,
                e.binary(CBinaryOperator::Add, left, right.clone()).unwrap(),
                right,
            )
            .unwrap(),
        5 => e
            .binary(
                CBinaryOperator::Add,
                e.binary(CBinaryOperator::Multiply, left, right).unwrap(),
                e.literal(CLiteral::F64(
                    portable_binary64::FiniteBinary64::from_bits(0xbff0000000000000).unwrap(),
                ))
                .unwrap(),
            )
            .unwrap(),
        _ => unreachable!(),
    }
}
