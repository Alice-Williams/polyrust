//! Flattened call DAG with exact signatures and explicit result temporaries.
use super::tests::key;
use crate::ast::*;
use portable_codegen::RelativeOutputPath;

pub(super) fn fixture(
    graph: &[Vec<usize>],
    extra_locals: usize,
    internal: bool,
) -> (CFrozenRegistry, CSourceFile) {
    scalar_fixture(
        graph,
        extra_locals,
        internal,
        &vec![1; graph.len()],
        CScalarType::I32,
    )
}

pub(super) fn scalar_fixture(
    graph: &[Vec<usize>],
    extra_locals: usize,
    internal: bool,
    arities: &[usize],
    kind: CScalarType,
) -> (CFrozenRegistry, CSourceFile) {
    assert_eq!(graph.len(), arities.len());
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("tests/calls.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let scalar = CObjectType::scalar(kind);
    let functions: Vec<_> = (0..graph.len())
        .map(|index| {
            registry
                .register_function(
                    &file,
                    key(&format!("call{index}")),
                    CFunctionType::new(
                        CReturnType::Value(CReturnValue::new(scalar.clone()).unwrap()),
                        vec![CParameterType::new(scalar.clone()).unwrap(); arities[index]],
                    ),
                )
                .unwrap()
        })
        .collect();
    let linkage = |index| {
        if internal && index != 0 {
            CLinkage::Internal
        } else {
            CLinkage::External
        }
    };
    let mut items: Vec<_> = functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            CFileItem::Declaration(
                CDeclarations::new(&registry, file.clone())
                    .unwrap()
                    .function_prototype(function.clone(), linkage(index))
                    .unwrap(),
            )
        })
        .collect();
    for (index, targets) in graph.iter().enumerate() {
        let function = &functions[index];
        let scope = registry
            .register_scope(function, None, key("root"))
            .unwrap();
        let parameters: Vec<_> = (0..arities[index])
            .map(|number| {
                registry
                    .register_parameter(
                        function,
                        number,
                        key(&format!("input{number}")),
                        CConstness::Unqualified,
                    )
                    .unwrap()
            })
            .collect();
        let expressions = CExpressions::new(&registry);
        let mut statements = vec![];
        // Explicitly use every parameter, including those the fixture ignores.
        for parameter in &parameters {
            let value = expressions
                .read(expressions.parameter(parameter.clone()).unwrap())
                .unwrap();
            statements.push(
                CStatements::new(&registry, function.clone())
                    .unwrap()
                    .discard(value)
                    .unwrap(),
            );
        }
        let mut value = match parameters.last() {
            Some(parameter) => expressions
                .read(expressions.parameter(parameter.clone()).unwrap())
                .unwrap(),
            None => expressions
                .literal(match kind {
                    CScalarType::I32 => CLiteral::Signed(CSignedLiteral::I32(0)),
                    CScalarType::Int => CLiteral::Signed(CSignedLiteral::Int(0)),
                    CScalarType::Bool => CLiteral::Bool(false),
                    _ => panic!("scalar call fixture type"),
                })
                .unwrap(),
        };
        for step in 0..extra_locals + targets.len() {
            let local = registry
                .register_local(&scope, key(&format!("step{step}")), scalar.clone())
                .unwrap();
            let expressions = CExpressions::new(&registry);
            if step >= extra_locals {
                value = expressions
                    .call_value(
                        expressions
                            .direct(functions[targets[step - extra_locals]].clone())
                            .unwrap(),
                        vec![value; arities[targets[step - extra_locals]]],
                    )
                    .unwrap();
            }
            let initial = expressions.expression_initializer(value).unwrap();
            statements.push(
                CStatements::new(&registry, function.clone())
                    .unwrap()
                    .declare(local.clone(), Some(initial))
                    .unwrap(),
            );
            value = expressions.read(expressions.local(local).unwrap()).unwrap();
        }
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        statements.push(ast.return_statement(Some(value)).unwrap());
        let body = ast.block(scope, statements).unwrap();
        items.push(CFileItem::Definition(
            CDeclarations::new(&registry, file.clone())
                .unwrap()
                .function_definition(function.clone(), linkage(index), parameters, body)
                .unwrap(),
        ));
    }
    let source = CDeclarations::new(&registry, file)
        .unwrap()
        .source_file(items)
        .unwrap();
    (registry.freeze(), source)
}
