//! Independent scalar readers; imported objects never enter owned inventories.
use super::{
    CDependencyApi, CDependencyConstant, CDependencyFunction, CDialect,
    constant_producer_tests::certify,
    owned_constant_fixture::{self, Fixture, Shape, key},
    owned_constant_tests::linked,
};
use crate::ast::*;
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Usage {
    Read,
    Unused,
    Difference,
}

pub(super) fn producer(shape: Shape) -> CDependencyApi {
    CDependencyApi::from_certificate(certify(&owned_constant_fixture::fixture(shape))).unwrap()
}

pub(super) fn fixture(
    crate_id: u64,
    values: &[CDependencyConstant],
    call: Option<CDependencyFunction>,
    usage: Usage,
) -> Fixture {
    named(crate_id, values, call, usage, None)
}

pub(super) fn named(
    crate_id: u64,
    values: &[CDependencyConstant],
    call: Option<CDependencyFunction>,
    usage: Usage,
    first_name: Option<&str>,
) -> Fixture {
    assert!(!values.is_empty());
    let id = |hash| RustDeclarationId {
        crate_id,
        definition_path_hash: hash,
    };
    let location = RustSourceLocation {
        file: format!("consumer_{crate_id}/src/lib.rs"),
        line: 1,
        column: 0,
    };
    let root = Arc::new(RustModuleDocumentation {
        declaration: id(1),
        parent: None,
        location: location.clone(),
        documentation: vec![],
    });
    let count = values.len() + usize::from(call.is_some());
    let mut names: Vec<_> = (0..count)
        .map(|index| format!("poly_import_{crate_id}_{index}"))
        .collect();
    if let Some(name) = first_name {
        names[0] = name.into();
    }
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
    let mut registry = CRegistry::new();
    let objects: Vec<_> = values
        .iter()
        .map(|value| registry.import_constant(value.clone()).unwrap())
        .collect();
    let imported_call = call.map(|call| registry.import_function(call).unwrap());
    let header = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("polyrust_constant_reader_{crate_id}.h"))
                .unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let source = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("polyrust_constant_reader_{crate_id}.c"))
                .unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let mut functions = vec![];
    let mut scopes = vec![];
    for (index, name) in names.iter().enumerate() {
        let signature = if index < values.len() {
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(values[index].read_type().clone()).unwrap()),
                vec![],
            )
        } else {
            CFunctionType::new(
                imported_call
                    .as_ref()
                    .unwrap()
                    .signature()
                    .return_type()
                    .clone(),
                vec![],
            )
        };
        assert!(signature.parameters().is_empty());
        let function = registry
            .register_function(
                &header,
                CDeclarationKey {
                    name: CIdentifier::new(name).unwrap(),
                    origin: CGeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                        declaration: id(10 + index as u64),
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
                signature,
            )
            .unwrap();
        scopes.push(
            registry
                .register_scope(&function, None, key("body"))
                .unwrap(),
        );
        functions.push(function);
    }
    let expressions = CExpressions::new(&registry);
    let declarations = CDeclarations::new(&registry, header).unwrap();
    let definitions = CDeclarations::new(&registry, source).unwrap();
    let mut header_items = vec![];
    let mut source_items = vec![];
    for (index, function) in functions.iter().enumerate() {
        let value = if index < values.len() {
            let read = || {
                expressions
                    .read(expressions.global(objects[index].clone()).unwrap())
                    .unwrap()
            };
            match usage {
                Usage::Unused => expressions
                    .literal(values[index].value().literal().unwrap())
                    .unwrap(),
                Usage::Difference
                    if values[index].read_type().kind()
                        != &CObjectTypeKind::Scalar(CScalarType::Bool) =>
                {
                    expressions
                        .binary(CBinaryOperator::Subtract, read(), read())
                        .unwrap()
                }
                Usage::Read | Usage::Difference => read(),
            }
        } else {
            expressions
                .call_value(
                    expressions
                        .direct(imported_call.as_ref().unwrap().clone())
                        .unwrap(),
                    imported_call
                        .as_ref()
                        .unwrap()
                        .signature()
                        .parameters()
                        .iter()
                        .map(|parameter| {
                            assert_eq!(
                                parameter.declared_type(),
                                &CObjectType::scalar(CScalarType::I32)
                            );
                            expressions
                                .literal(CLiteral::Signed(CSignedLiteral::I32(42)))
                                .unwrap()
                        })
                        .collect(),
                )
                .unwrap()
        };
        let statements = CStatements::new(&registry, function.clone()).unwrap();
        let body = statements
            .block(
                scopes[index].clone(),
                vec![statements.return_statement(Some(value)).unwrap()],
            )
            .unwrap();
        header_items.push(CFileItem::Declaration(
            declarations
                .function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
        ));
        source_items.push(CFileItem::Definition(
            definitions
                .function_definition(function.clone(), CLinkage::External, vec![], body)
                .unwrap(),
        ));
    }
    let files = vec![
        declarations.source_file(header_items).unwrap(),
        definitions.source_file(source_items).unwrap(),
    ];
    Fixture {
        registry: registry.freeze(),
        files,
        objects,
        functions,
    }
}

pub(super) fn api(fixture: &Fixture) -> CDependencyApi {
    CDependencyApi::from_certificate(certify_resolved_package(&CDialect, linked(fixture)).unwrap())
        .unwrap()
}
