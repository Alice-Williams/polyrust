use super::*;

pub(in crate::tests) const WIDTHS: [JavaPrimitive; 2] = [JavaPrimitive::Int, JavaPrimitive::Long];
pub(in crate::tests) fn literal(width: JavaPrimitive, value: i32) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(width),
        match width {
            JavaPrimitive::Int => JavaLiteral::I32(value),
            JavaPrimitive::Long => JavaLiteral::I64(i64::from(value)),
            _ => unreachable!(),
        },
    )
}
pub(in crate::tests) fn add(width: JavaPrimitive, left: JavaExpr, right: JavaExpr) -> JavaExpr {
    binary(JavaBinaryOperator::Add, width, left, right)
}
pub(in crate::tests) fn binary(
    operator: JavaBinaryOperator,
    width: JavaPrimitive,
    left: JavaExpr,
    right: JavaExpr,
) -> JavaExpr {
    JavaExpr {
        ty: JavaType::primitive(width),
        precedence: JavaPrecedence::Additive,
        kind: JavaExprKind::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        },
    }
}
pub(in crate::tests) fn function(index: usize, value: JavaExpr) -> f::Function {
    let ty = JavaType::primitive(WIDTHS[index]);
    f::Function {
        hash: 10 + index as u64,
        public: true,
        name: f::name(&format!("addition{index}")),
        parameters: ["left", "right"]
            .into_iter()
            .map(|name| JavaParameter {
                ty: ty.clone(),
                name: f::name(name),
                final_parameter: true,
            })
            .collect(),
        result: ty,
        body: JavaBlock::new(vec![JavaStmt::Return(Some(value))]),
    }
}
pub(in crate::tests) fn chain() -> Vec<JavaDependencyApi> {
    chain_with_operator(JavaBinaryOperator::Add)
}
pub(in crate::tests) fn chain_with_operator(
    operator: JavaBinaryOperator,
) -> Vec<JavaDependencyApi> {
    let make_function = |index, value| {
        let mut declaration = function(index, value);
        let prefix = match operator {
            JavaBinaryOperator::Add => "addition",
            JavaBinaryOperator::Subtract => "subtraction",
            _ => panic!("shared fixture requires an additive integer operation"),
        };
        declaration.name = f::name(&format!("{prefix}{index}"));
        declaration
    };
    let functions = WIDTHS
        .into_iter()
        .enumerate()
        .map(|(index, width)| {
            let ty = JavaType::primitive(width);
            make_function(
                index,
                binary(
                    operator,
                    width,
                    JavaExpr::local(ty.clone(), f::name("left")),
                    JavaExpr::local(ty, f::name("right")),
                ),
            )
        })
        .collect();
    let producer =
        JavaDependencyApi::from_certificate(f::certify(f::package(801, functions))).unwrap();
    let mut scope = JavaDependencyScope::new();
    let functions = producer
        .functions()
        .cloned()
        .enumerate()
        .map(|(index, target)| {
            let ty = JavaType::primitive(WIDTHS[index]);
            let (next, callable) = std::mem::take(&mut scope).import(target);
            scope = next;
            make_function(
                index,
                JavaExpr {
                    ty: ty.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Call {
                        callable: JavaCallableRef::Dependency(callable),
                        receiver: None,
                        arguments: ["left", "right"]
                            .into_iter()
                            .map(|name| JavaExpr::local(ty.clone(), f::name(name)))
                            .collect(),
                    },
                },
            )
        })
        .collect();
    let consumer = JavaDependencyApi::from_certificate(f::certify(f::package_with_dependencies(
        802,
        functions,
        scope.finish(),
    )))
    .unwrap();
    vec![producer, consumer]
}
pub(in crate::tests) fn admitted(package: TargetAstPackage<JavaDialect>) -> bool {
    let Ok(verified) = verify_unresolved_package(&JavaDialect, package) else {
        return false;
    };
    let Ok(linked) = TargetLinker::new(JavaDialect).link_ast(&verified) else {
        return false;
    };
    let Ok(ready) = certify_resolved_package(&JavaDialect, linked) else {
        return false;
    };
    JavaDependencyApi::from_certificate(ready).is_ok()
}
