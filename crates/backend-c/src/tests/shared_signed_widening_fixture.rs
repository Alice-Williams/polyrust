//! Source-owned unary widening packages with a materialized original operand.
use crate::ast::*;
use crate::dialect::CDependencyFunction;
use crate::dialect::shared::dependency_fixture as f;
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy)]
pub(super) enum Body {
    Widen,
    Forward,
    Zero,
    NestedComplement(usize),
    UnadmittedDivide,
}

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
    }
}

pub(super) fn build(
    crate_id: u64,
    dependency: Option<CDependencyFunction>,
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
            BTreeMap::from([(
                RustExportName {
                    namespace: RustExportNamespace::Value,
                    name: "widen".into(),
                },
                RustExportTarget::Declaration(id(10)),
            )]),
        )]),
        module_ancestries: BTreeMap::from([(id(1), vec![module.clone()].into())]),
    });
    let mut registry = CRegistry::new();
    let imported = dependency
        .map(|dependency| registry.import_function(dependency).unwrap())
        .into_iter()
        .collect::<Vec<_>>();
    let mut file = |extension, role| {
        registry
            .register_file(CFileKey {
                path: RelativeOutputPath::new(format!("polyrust_widening_{crate_id}.{extension}"))
                    .unwrap(),
                role,
            })
            .unwrap()
    };
    let header = file("h", CFileRole::GeneratedPublicHeader);
    let implementation = file("c", CFileRole::GeneratedSource);
    let function = registry
        .register_function(
            &header,
            CDeclarationKey {
                name: CIdentifier::new(&format!("poly_widen_{crate_id}")).unwrap(),
                origin: CGeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                    declaration: id(10),
                    node: RustSourceNode::Declaration,
                    module: id(1),
                    location,
                    visibility: RustVisibility::Public,
                    externally_reachable: true,
                    documentation: vec![],
                    module_ancestors: vec![module].into(),
                    crate_exports: exports,
                })),
            },
            CFunctionType::new(
                CReturnType::Value(
                    CReturnValue::new(CObjectType::scalar(CScalarType::I64)).unwrap(),
                ),
                vec![CParameterType::new(CObjectType::scalar(CScalarType::I32)).unwrap()],
            ),
        )
        .unwrap();
    let parameter = registry
        .register_parameter(&function, 0, key("input"), CConstness::Unqualified)
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("body"))
        .unwrap();
    let local = (!matches!(mode, Body::Forward)).then(|| {
        registry
            .register_local(
                &scope,
                key("operand"),
                CObjectType::scalar(CScalarType::I32),
            )
            .unwrap()
    });
    let e = CExpressions::new(&registry);
    let s = CStatements::new(&registry, function.clone()).unwrap();
    let input = e.read(e.parameter(parameter.clone()).unwrap()).unwrap();
    let operand = if let Some(target) = imported.first() {
        e.call_value(e.direct(target.clone()).unwrap(), vec![input])
            .unwrap()
    } else {
        input
    };
    let mut statements = Vec::new();
    let value = if matches!(mode, Body::Forward) {
        operand
    } else {
        let local = local.unwrap();
        statements.push(
            s.declare(
                local.clone(),
                Some(e.expression_initializer(operand).unwrap()),
            )
            .unwrap(),
        );
        let mut operand = e.read(e.local(local).unwrap()).unwrap();
        if let Body::NestedComplement(depth) = mode {
            for _ in 0..depth {
                operand = e.unary(CUnaryOperator::BitNot, operand).unwrap();
                operand = e.numeric_conversion(CScalarType::I32, operand).unwrap();
            }
        }
        if matches!(mode, Body::UnadmittedDivide) {
            operand = e
                .binary(
                    CBinaryOperator::Divide,
                    operand,
                    e.literal(CLiteral::Signed(CSignedLiteral::I32(1))).unwrap(),
                )
                .unwrap();
            operand = e.numeric_conversion(CScalarType::I32, operand).unwrap();
        }
        if matches!(mode, Body::Zero) {
            statements.push(s.discard(operand).unwrap());
            e.literal(CLiteral::Signed(CSignedLiteral::I64(0))).unwrap()
        } else {
            e.numeric_conversion(CScalarType::I64, operand).unwrap()
        }
    };
    statements.push(s.return_statement(Some(value)).unwrap());
    let body = s.block(scope, statements).unwrap();
    let declaration = CDeclarations::new(&registry, header).unwrap();
    let header = declaration
        .source_file(vec![CFileItem::Declaration(
            declaration
                .function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
        )])
        .unwrap();
    let declaration = CDeclarations::new(&registry, implementation).unwrap();
    let implementation = declaration
        .source_file(vec![CFileItem::Definition(
            declaration
                .function_definition(function.clone(), CLinkage::External, vec![parameter], body)
                .unwrap(),
        )])
        .unwrap();
    f::Fixture {
        registry: registry.freeze(),
        files: vec![header, implementation],
        functions: vec![function],
        imported,
    }
}
