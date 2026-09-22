use super::*;

pub(super) const WIDTHS: [JavaPrimitive; 2] = [JavaPrimitive::Int, JavaPrimitive::Long];
pub(super) fn literal(width: JavaPrimitive, value: i32) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(width),
        match width {
            JavaPrimitive::Int => JavaLiteral::I32(value),
            JavaPrimitive::Long => JavaLiteral::I64(i64::from(value)),
            _ => unreachable!(),
        },
    )
}
pub(super) fn add(width: JavaPrimitive, left: JavaExpr, right: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: JavaType::primitive(width),
        precedence: JavaPrecedence::Additive,
        kind: JavaExprKind::Binary {
            operator: JavaBinaryOperator::Add,
            left: Box::new(left),
            right: Box::new(right),
        },
    }
}
pub(super) fn function(index: usize, value: JavaExpr) -> f::Function {
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
pub(super) fn chain() -> Vec<JavaDependencyApi> {
    let functions = WIDTHS
        .into_iter()
        .enumerate()
        .map(|(index, width)| {
            let ty = JavaType::primitive(width);
            function(
                index,
                add(
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
            function(
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
pub(super) fn admitted(package: TargetAstPackage<JavaDialect>) -> bool {
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
