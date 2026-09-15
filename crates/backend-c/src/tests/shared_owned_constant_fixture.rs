//! Registered const scalar objects with optional ordinary read functions.
use crate::ast::*;
use portable_codegen::{
    RelativeOutputPath, RustDeclarationId, RustExportName, RustExportNamespace, RustExportTarget,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Shape {
    ConstantsOnly,
    Mixed,
}

pub(super) struct Fixture {
    pub registry: CFrozenRegistry,
    pub files: Vec<CSourceFile>,
    pub objects: Vec<CObjectRef>,
    pub functions: Vec<CFunctionRef>,
}

pub(super) fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter),
    }
}

pub(super) fn fixture(shape: Shape) -> Fixture {
    build(shape, ConstantOrigins::RustSource)
}

#[derive(Clone, Copy)]
enum ConstantOrigins {
    RustSource,
    Synthesized,
}

pub(super) fn synthesized_constants_fixture() -> Fixture {
    build(Shape::Mixed, ConstantOrigins::Synthesized)
}

fn build(shape: Shape, origins: ConstantOrigins) -> Fixture {
    let literals = [
        ("false_value", CLiteral::Bool(false)),
        ("true_value", CLiteral::Bool(true)),
        ("i32_min", CLiteral::Signed(CSignedLiteral::I32(i32::MIN))),
        ("i32_max", CLiteral::Signed(CSignedLiteral::I32(i32::MAX))),
        ("i64_min", CLiteral::Signed(CSignedLiteral::I64(i64::MIN))),
        ("i64_max", CLiteral::Signed(CSignedLiteral::I64(i64::MAX))),
        (
            "wide_value",
            CLiteral::Signed(CSignedLiteral::I64(9_007_199_254_740_993)),
        ),
        ("computed_value", CLiteral::Signed(CSignedLiteral::I32(62))),
    ];
    let (base, _) = super::package_source_fixture::origins();
    let id = |hash| RustDeclarationId {
        crate_id: base.declaration.crate_id,
        definition_path_hash: hash,
    };
    let mut exports = (*base.crate_exports).clone();
    let entries = exports.modules.get_mut(&exports.root).unwrap();
    entries.clear();
    for (index, (name, _)) in literals.iter().enumerate() {
        if matches!(origins, ConstantOrigins::RustSource) {
            entries.insert(
                RustExportName {
                    namespace: RustExportNamespace::Value,
                    name: (*name).into(),
                },
                RustExportTarget::Declaration(id(10 + index as u64)),
            );
        }
        if shape == Shape::Mixed {
            entries.insert(
                RustExportName {
                    namespace: RustExportNamespace::Value,
                    name: format!("read_{name}"),
                },
                RustExportTarget::Declaration(id(100 + index as u64)),
            );
        }
    }
    let exports = Arc::new(exports);
    let origin = |name: &str, hash| {
        let mut origin = base.clone();
        origin.declaration = id(hash);
        origin.documentation = vec![format!("Documentation for {name}.")];
        origin.crate_exports = exports.clone();
        CGeneratedOrigin::RustSource(Arc::new(origin))
    };
    let mut registry = CRegistry::new();
    let header = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("polyrust_constants.h").unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let source = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("constants.c").unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let mut objects = Vec::new();
    let mut functions = Vec::new();
    let mut scopes = Vec::new();
    for (index, (name, literal)) in literals.iter().enumerate() {
        let ty = CExpressions::new(&registry)
            .literal(literal.clone())
            .unwrap()
            .ty()
            .clone();
        objects.push(
            registry
                .register_object(
                    &header,
                    CDeclarationKey {
                        origin: match origins {
                            ConstantOrigins::RustSource => origin(name, 10 + index as u64),
                            ConstantOrigins::Synthesized => key(name).origin,
                        },
                        ..key(name)
                    },
                    ty.clone().with_constness(CConstness::Const).unwrap(),
                )
                .unwrap(),
        );
        if shape == Shape::Mixed {
            let function = registry
                .register_function(
                    &header,
                    CDeclarationKey {
                        origin: origin(&format!("read_{name}"), 100 + index as u64),
                        ..key(&format!("read_{name}"))
                    },
                    CFunctionType::new(CReturnType::Value(CReturnValue::new(ty).unwrap()), vec![]),
                )
                .unwrap();
            scopes.push(
                registry
                    .register_scope(&function, None, key(&format!("body_{name}")))
                    .unwrap(),
            );
            functions.push(function);
        }
    }
    let expressions = CExpressions::new(&registry);
    let declarations = CDeclarations::new(&registry, header).unwrap();
    let definitions = CDeclarations::new(&registry, source).unwrap();
    let mut header_items = Vec::new();
    let mut source_items = Vec::new();
    for ((_, literal), object) in literals.iter().zip(&objects) {
        header_items.push(CFileItem::Declaration(
            declarations.object_declaration(object.clone()).unwrap(),
        ));
        source_items.push(CFileItem::Definition(
            definitions
                .object_definition(
                    object.clone(),
                    CLinkage::External,
                    expressions
                        .expression_initializer(expressions.literal(literal.clone()).unwrap())
                        .unwrap(),
                )
                .unwrap(),
        ));
    }
    for ((function, scope), object) in functions.iter().zip(scopes).zip(&objects) {
        header_items.push(CFileItem::Declaration(
            declarations
                .function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
        ));
        let statements = CStatements::new(&registry, function.clone()).unwrap();
        let value = expressions
            .read(expressions.global(object.clone()).unwrap())
            .unwrap();
        let body = statements
            .block(
                scope,
                vec![statements.return_statement(Some(value)).unwrap()],
            )
            .unwrap();
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
