//! One owned scalar constant beside a call into a certified dependency.
use super::{
    CDependencyApi, CDependencyFunction, CDialect, dependency_fixture,
    owned_constant_fixture::{Fixture, key},
};
use crate::ast::*;
use portable_codegen::{RelativeOutputPath, certify_resolved_package};

pub(super) fn owner() -> CDependencyApi {
    let input = dependency_fixture::named(
        70,
        &["poly_selected", "poly_other_export"],
        &[],
        &[None, None],
    );
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, dependency_fixture::linked(&input)).unwrap(),
    )
    .unwrap()
}

pub(super) fn selected(owner: &CDependencyApi) -> CDependencyFunction {
    owner.functions().next().unwrap().clone()
}

pub(super) fn middle(owner: &CDependencyApi) -> CDependencyApi {
    let input = dependency_fixture::named(80, &["poly_middle"], &[selected(owner)], &[Some(0)]);
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, dependency_fixture::linked(&input)).unwrap(),
    )
    .unwrap()
}

pub(super) fn consumer(name: &str, dependency: CDependencyFunction, call: bool) -> Fixture {
    with_call(
        name,
        "consumer_read",
        dependency,
        &[CLiteral::Signed(CSignedLiteral::I32(42))],
        call,
    )
}

pub(super) fn with_call(
    name: &str,
    function_name: &str,
    dependency: CDependencyFunction,
    arguments: &[CLiteral],
    call: bool,
) -> Fixture {
    let mut registry = CRegistry::new();
    let imported = registry.import_function(dependency).unwrap();
    let header = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("polyrust_constant_consumer.h").unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let source = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("constant_consumer.c").unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let ty = CObjectType::scalar(CScalarType::I32);
    let object = registry
        .register_object(
            &header,
            key(name),
            ty.clone().with_constness(CConstness::Const).unwrap(),
        )
        .unwrap();
    let function = registry
        .register_function(
            &header,
            key(function_name),
            CFunctionType::new(CReturnType::Value(CReturnValue::new(ty).unwrap()), vec![]),
        )
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("body"))
        .unwrap();
    let expressions = CExpressions::new(&registry);
    let literal = || {
        expressions
            .literal(CLiteral::Signed(CSignedLiteral::I32(42)))
            .unwrap()
    };
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let value = if call {
        expressions
            .call_value(
                expressions.direct(imported).unwrap(),
                arguments
                    .iter()
                    .map(|value| expressions.literal(value.clone()).unwrap())
                    .collect(),
            )
            .unwrap()
    } else {
        literal()
    };
    let body = statements
        .block(
            scope,
            vec![statements.return_statement(Some(value)).unwrap()],
        )
        .unwrap();
    let declarations = CDeclarations::new(&registry, header).unwrap();
    let definitions = CDeclarations::new(&registry, source).unwrap();
    let files = vec![
        declarations
            .source_file(vec![
                CFileItem::Declaration(declarations.object_declaration(object.clone()).unwrap()),
                CFileItem::Declaration(
                    declarations
                        .function_prototype(function.clone(), CLinkage::External)
                        .unwrap(),
                ),
            ])
            .unwrap(),
        definitions
            .source_file(vec![
                CFileItem::Definition(
                    definitions
                        .object_definition(
                            object.clone(),
                            CLinkage::External,
                            expressions.expression_initializer(literal()).unwrap(),
                        )
                        .unwrap(),
                ),
                CFileItem::Definition(
                    definitions
                        .function_definition(function.clone(), CLinkage::External, vec![], body)
                        .unwrap(),
                ),
            ])
            .unwrap(),
    ];
    Fixture {
        registry: registry.freeze(),
        files,
        objects: vec![object],
        functions: vec![function],
    }
}
