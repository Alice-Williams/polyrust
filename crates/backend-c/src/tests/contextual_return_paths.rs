//! Return coverage across branch and zero-iteration edges, not just direct returns.
use super::contextual_reconstruction::{int, key};
use super::*;

#[test]
fn nonvoid_branches_and_zero_iteration_exits_each_need_a_return() {
    for loop_body in [false, true] {
        for cover_exit in [false, true] {
            let mut registry = CRegistry::new();
            let file = registry
                .register_file(CFileKey {
                    path: portable_codegen::RelativeOutputPath::new("return.c").unwrap(),
                    role: CFileRole::TestSource,
                })
                .unwrap();
            let function = registry
                .register_function(
                    &file,
                    key("run"),
                    CFunctionType::new(
                        CReturnType::Value(
                            CReturnValue::new(CObjectType::scalar(CScalarType::I32)).unwrap(),
                        ),
                        vec![CParameterType::new(CObjectType::scalar(CScalarType::Bool)).unwrap()],
                    ),
                )
                .unwrap();
            let parameter = registry
                .register_parameter(&function, 0, key("condition"), CConstness::Unqualified)
                .unwrap();
            let root = registry
                .register_scope(&function, None, key("root"))
                .unwrap();
            let child = registry
                .register_scope(&function, Some(&root), key("child"))
                .unwrap();
            let mut declarations = Vec::new();
            let statement = if loop_body {
                let identity = registry.register_loop(&root, key("iteration")).unwrap();
                let size = CObjectType::scalar(CScalarType::Size);
                let counter = registry
                    .register_local(&root, key("counter"), size.clone())
                    .unwrap();
                let bound = registry
                    .register_local(
                        &root,
                        key("bound"),
                        size.with_constness(CConstness::Const).unwrap(),
                    )
                    .unwrap();
                let values = CExpressions::new(&registry);
                let ast = CStatements::new(&registry, function.clone()).unwrap();
                let literal = |n| {
                    values
                        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(n)))
                        .unwrap()
                };
                declarations.push(
                    ast.declare(
                        counter.clone(),
                        Some(values.expression_initializer(literal(0)).unwrap()),
                    )
                    .unwrap(),
                );
                declarations.push(
                    ast.declare(
                        bound.clone(),
                        Some(values.expression_initializer(literal(1)).unwrap()),
                    )
                    .unwrap(),
                );
                let condition = values
                    .numeric_conversion(
                        CScalarType::Bool,
                        values
                            .binary(
                                CBinaryOperator::Less,
                                values.read(values.local(counter.clone()).unwrap()).unwrap(),
                                values.read(values.local(bound.clone()).unwrap()).unwrap(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                ast.counted_loop(
                    identity,
                    counter,
                    bound,
                    condition,
                    ast.block(
                        child,
                        vec![ast.return_statement(Some(int(&values))).unwrap()],
                    )
                    .unwrap(),
                )
                .unwrap()
            } else {
                let otherwise = registry
                    .register_scope(&function, Some(&root), key("otherwise"))
                    .unwrap();
                let values = CExpressions::new(&registry);
                let ast = CStatements::new(&registry, function.clone()).unwrap();
                let returned = ast.return_statement(Some(int(&values))).unwrap();
                ast.if_statement(
                    values
                        .read(values.parameter(parameter.clone()).unwrap())
                        .unwrap(),
                    ast.block(child, vec![returned.clone()]).unwrap(),
                    ast.block(otherwise, if cover_exit { vec![returned] } else { vec![] })
                        .unwrap(),
                )
                .unwrap()
            };
            declarations.push(statement);
            let values = CExpressions::new(&registry);
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            if loop_body && cover_exit {
                declarations.push(ast.return_statement(Some(int(&values))).unwrap());
            }
            let body = ast.block(root, declarations).unwrap();
            let files = CDeclarations::new(&registry, file).unwrap();
            let source = files
                .source_file(vec![CFileItem::Definition(
                    files
                        .function_definition(function, CLinkage::External, vec![parameter], body)
                        .unwrap(),
                )])
                .unwrap();
            assert_eq!(
                registry.check_context(&[source]),
                if cover_exit {
                    Ok(())
                } else {
                    Err(CContextError::MissingReturn)
                },
                "loop={loop_body}, cover_exit={cover_exit}"
            );
        }
    }
}
