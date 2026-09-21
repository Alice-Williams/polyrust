//! Original-owner remainder fixture; compiler-owned Rust witnesses are separate.
use super::*;
pub(super) fn literal(bits: u64) -> JavaExpr {
    JavaExpr::literal(
        double(),
        JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(bits).unwrap()),
    )
}
pub(super) fn binary(operator: JavaBinaryOperator, left: JavaExpr, right: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: double(),
        precedence: match operator {
            JavaBinaryOperator::Add | JavaBinaryOperator::Subtract => JavaPrecedence::Additive,
            _ => JavaPrecedence::Multiplicative,
        },
        kind: JavaExprKind::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        },
    }
}
pub(super) fn expression(index: usize, left: JavaExpr, right: JavaExpr) -> JavaExpr {
    assert_eq!(index, 0);
    binary(JavaBinaryOperator::Remainder, left, right)
}
pub(super) fn parameters() -> Vec<JavaParameter> {
    ["left", "right"]
        .into_iter()
        .map(|name| JavaParameter {
            ty: double(),
            name: f::name(name),
            final_parameter: true,
        })
        .collect()
}
pub(super) fn function(index: usize, body: JavaBlock) -> f::Function {
    f::Function {
        hash: 10 + index as u64,
        public: true,
        name: f::name(&format!("remainder{index}")),
        parameters: parameters(),
        result: double(),
        body,
    }
}
fn call(
    scope: &mut JavaDependencyScope,
    function: JavaDependencyFunction,
    arguments: Vec<JavaExpr>,
) -> JavaExpr {
    let (next, callable) = std::mem::take(scope).import(function);
    *scope = next;
    JavaExpr {
        ty: double(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Dependency(callable),
            receiver: None,
            arguments,
        },
    }
}
pub(super) fn owner(
    crate_id: u64,
    dependency: &JavaDependencyApi,
    forward: bool,
) -> JavaDependencyApi {
    let imports: Vec<_> = dependency.functions().cloned().collect();
    let mut scope = JavaDependencyScope::new();
    let declarations = (0..1)
        .map(|index| {
            let left = JavaExpr::local(double(), f::name("left"));
            let right = JavaExpr::local(double(), f::name("right"));
            let body = if forward {
                JavaBlock::new(vec![JavaStmt::Return(Some(call(
                    &mut scope,
                    imports[index].clone(),
                    vec![left, right],
                )))])
            } else {
                let values = [left, right]
                    .into_iter()
                    .enumerate()
                    .map(|(side, value)| JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: double(),
                        name: f::name(if side == 0 { "leftValue" } else { "rightValue" }),
                        value: Some(call(&mut scope, imports[side].clone(), vec![value])),
                    })
                    .collect::<Vec<_>>();
                let mut statements = values;
                statements.push(JavaStmt::Return(Some(expression(
                    index,
                    JavaExpr::local(double(), f::name("leftValue")),
                    JavaExpr::local(double(), f::name("rightValue")),
                ))));
                JavaBlock::new(statements)
            };
            function(index, body)
        })
        .collect();
    JavaDependencyApi::from_certificate(f::certify(f::package_with_dependencies(
        crate_id,
        declarations,
        scope.finish(),
    )))
    .unwrap()
}
pub(super) fn chain() -> Vec<JavaDependencyApi> {
    let declarations = ["leftIdentity", "rightIdentity"]
        .into_iter()
        .enumerate()
        .map(|(index, name)| f::Function {
            hash: 10 + index as u64,
            public: true,
            name: f::name(name),
            parameters: vec![JavaParameter {
                ty: double(),
                name: f::name("input"),
                final_parameter: true,
            }],
            result: double(),
            body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::local(
                double(),
                f::name("input"),
            )))]),
        })
        .collect();
    let leaf =
        JavaDependencyApi::from_certificate(f::certify(f::package(601, declarations))).unwrap();
    let middle = owner(602, &leaf, false);
    let root = owner(603, &middle, true);
    vec![leaf, middle, root]
}
