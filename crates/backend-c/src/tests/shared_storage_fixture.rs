//! Many simultaneously scoped locals and by-value aggregate copies.
use super::tests::key;
use crate::ast::*;
use portable_codegen::RelativeOutputPath;

pub(super) fn fixture(count: usize, fields: usize) -> (CFrozenRegistry, CSourceFile) {
    assert!(count > 0);
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("tests/storage.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let scalar = CObjectType::scalar(CScalarType::I32);
    let function = registry
        .register_function(
            &file,
            key("identity"),
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(scalar.clone()).unwrap()),
                vec![CParameterType::new(scalar.clone()).unwrap()],
            ),
        )
        .unwrap();
    let parameter = registry
        .register_parameter(&function, 0, key("input"), CConstness::Unqualified)
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("root"))
        .unwrap();
    let record = if fields > 0 {
        let record = registry.declare_struct(&file, key("record")).unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        let members = (0..fields)
            .map(|index| {
                registry
                    .register_member(&owner, key(&format!("field{index}")), scalar.clone())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        registry.define_aggregate(&owner, members.clone()).unwrap();
        Some((record, owner, members))
    } else {
        None
    };
    let ty = record
        .as_ref()
        .map(|(record, _, _)| CObjectType::structure(record.clone()))
        .unwrap_or(scalar);
    let locals = (0..count)
        .map(|index| {
            registry
                .register_local(&scope, key(&format!("local{index}")), ty.clone())
                .unwrap()
        })
        .collect::<Vec<_>>();
    let expressions = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let input = expressions
        .read(expressions.parameter(parameter.clone()).unwrap())
        .unwrap();
    let initial = if let Some((record, _, members)) = &record {
        expressions
            .struct_initializer(
                record.clone(),
                members
                    .iter()
                    .map(|member| {
                        (
                            member.clone(),
                            expressions.expression_initializer(input.clone()).unwrap(),
                        )
                    })
                    .collect(),
            )
            .unwrap()
    } else {
        expressions.expression_initializer(input).unwrap()
    };
    let mut body = vec![
        statements
            .declare(locals[0].clone(), Some(initial))
            .unwrap(),
    ];
    for pair in locals.windows(2) {
        let copied = expressions
            .read(expressions.local(pair[0].clone()).unwrap())
            .unwrap();
        body.push(
            statements
                .declare(
                    pair[1].clone(),
                    Some(expressions.expression_initializer(copied).unwrap()),
                )
                .unwrap(),
        );
    }
    let last = expressions.local(locals.last().unwrap().clone()).unwrap();
    let last = if let Some((_, _, members)) = &record {
        expressions
            .member(last, members.last().unwrap().clone())
            .unwrap()
    } else {
        last
    };
    body.push(
        statements
            .return_statement(Some(expressions.read(last).unwrap()))
            .unwrap(),
    );
    let body = statements.block(scope, body).unwrap();
    let declarations = CDeclarations::new(&registry, file).unwrap();
    let mut items = vec![];
    if let Some((_, owner, _)) = record {
        items.push(CFileItem::Declaration(
            declarations.aggregate(owner).unwrap(),
        ));
    }
    items.push(CFileItem::Declaration(
        declarations
            .function_prototype(function.clone(), CLinkage::External)
            .unwrap(),
    ));
    items.push(CFileItem::Definition(
        declarations
            .function_definition(function, CLinkage::External, vec![parameter], body)
            .unwrap(),
    ));
    let source = declarations.source_file(items).unwrap();
    (registry.freeze(), source)
}
