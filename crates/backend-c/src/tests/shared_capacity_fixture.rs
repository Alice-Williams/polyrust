//! Candidate boundary probes. These are not yet certified admission limits.
use super::tests::key;
use crate::ast::{
    CAggregateRef, CComment, CConstness, CDeclarations, CExpressions, CFileItem, CFileKey,
    CFileRole, CFrozenRegistry, CFunctionType, CLinkage, CLiteral, CObjectType, CParameterType,
    CRegistry, CReturnType, CReturnValue, CScalarType, CSignedLiteral, CSourceFile, CStatements,
};
use portable_codegen::RelativeOutputPath;

#[derive(Clone, Copy, Debug)]
pub(super) enum Shape {
    Parameters(usize),
    Fields(usize),
    NestedBlocks(usize),
    IdentifierBytes(usize),
    CommentBytes(usize),
    Combined,
    Conversions(usize),
}

pub(super) fn fixture(shape: Shape) -> (CFrozenRegistry, CSourceFile) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("tests/capacity.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let scalar = CObjectType::scalar(CScalarType::I32);
    let count = match shape {
        Shape::Parameters(count) => count,
        Shape::Combined => 127,
        _ => 1,
    };
    let function = registry
        .register_function(
            &file,
            key("identity"),
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(scalar.clone()).unwrap()),
                (0..count)
                    .map(|_| CParameterType::new(scalar.clone()).unwrap())
                    .collect(),
            ),
        )
        .unwrap();
    let parameters = (0..count)
        .map(|index| {
            let identifier_bytes = match shape {
                Shape::IdentifierBytes(bytes) => Some(bytes),
                Shape::Combined => Some(256),
                _ => None,
            };
            let name = if let Some(bytes) = identifier_bytes {
                // The shared linker adds the five-byte poly_ prefix.
                format!("{}{:03}", "a".repeat(bytes.checked_sub(8).unwrap()), index)
            } else {
                format!("argument{index}")
            };
            registry
                .register_parameter(&function, index, key(&name), CConstness::Unqualified)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let root = registry
        .register_scope(&function, None, key("root"))
        .unwrap();
    let mut scopes = vec![root];
    let nesting = match shape {
        Shape::NestedBlocks(count) => count,
        Shape::Combined => 40,
        _ => 0,
    };
    {
        let count = nesting;
        for index in 0..count {
            let scope = registry
                .register_scope(&function, scopes.last(), key(&format!("scope{index}")))
                .unwrap();
            scopes.push(scope);
        }
    }
    let mut aggregate = None;
    let fields = match shape {
        Shape::Fields(count) => count,
        Shape::Combined => 256,
        _ => 0,
    };
    if fields > 0 {
        let count = fields;
        let record = registry.declare_struct(&file, key("record")).unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        let members = (0..count)
            .map(|index| {
                registry
                    .register_member(&owner, key(&format!("field{index}")), scalar.clone())
                    .unwrap()
            })
            .collect::<Vec<_>>();
        registry.define_aggregate(&owner, members.clone()).unwrap();
        let local = registry
            .register_local(
                scopes.last().unwrap(),
                key("value"),
                CObjectType::structure(record.clone()),
            )
            .unwrap();
        aggregate = Some((record, owner, members, local));
    }
    let expressions = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let input = match parameters.first() {
        Some(parameter) => expressions
            .read(expressions.parameter(parameter.clone()).unwrap())
            .unwrap(),
        None => expressions
            .literal(CLiteral::Signed(CSignedLiteral::I32(42)))
            .unwrap(),
    };
    let mut body = Vec::new();
    for parameter in &parameters {
        let value = expressions
            .read(expressions.parameter(parameter.clone()).unwrap())
            .unwrap();
        body.push(statements.discard(value).unwrap());
    }
    let mut result = if let Some((record, _, members, local)) = &aggregate {
        let fields = members
            .iter()
            .map(|member| {
                (
                    member.clone(),
                    expressions.expression_initializer(input.clone()).unwrap(),
                )
            })
            .collect();
        let initializer = expressions
            .struct_initializer(record.clone(), fields)
            .unwrap();
        body.push(
            statements
                .declare(local.clone(), Some(initializer))
                .unwrap(),
        );
        expressions
            .read(
                expressions
                    .member(
                        expressions.local(local.clone()).unwrap(),
                        members.last().unwrap().clone(),
                    )
                    .unwrap(),
            )
            .unwrap()
    } else {
        input
    };
    if let Shape::Conversions(count) = shape {
        for _ in 0..count {
            result = expressions
                .numeric_conversion(CScalarType::I32, result)
                .unwrap();
        }
    }
    body.push(statements.return_statement(Some(result)).unwrap());
    let mut body = statements.block(scopes.pop().unwrap(), body).unwrap();
    while let Some(scope) = scopes.pop() {
        body = statements
            .block(scope, vec![statements.nested_block(body).unwrap()])
            .unwrap();
    }
    let declarations = CDeclarations::new(&registry, file).unwrap();
    let mut items = vec![];
    let comments = match shape {
        Shape::CommentBytes(bytes) => bytes,
        Shape::Combined => 1024 * 1024,
        _ => 0,
    };
    if comments > 0 {
        let bytes = comments;
        items.push(CFileItem::Comment(CComment::new(&"x".repeat(bytes))));
    }
    if let Some((_, owner, _, _)) = aggregate {
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
            .function_definition(function, CLinkage::External, parameters, body)
            .unwrap(),
    ));
    let source = declarations.source_file(items).unwrap();
    (registry.freeze(), source)
}
