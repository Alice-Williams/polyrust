//! Independent Java target fixtures; Int is storage, not Unicode validation.
use super::*;

pub(super) const BOUNDARIES: [char; 19] = [
    '\u{0}',
    '\u{1}',
    '\u{7f}',
    '\u{80}',
    '\u{ff}',
    '\u{100}',
    '\u{378}',
    '\u{7ff}',
    '\u{800}',
    '\u{d7ff}',
    '\u{e000}',
    '\u{fdd0}',
    '\u{fffe}',
    '\u{ffff}',
    '\u{10000}',
    '\u{1f980}',
    '\u{f0000}',
    '\u{10fffe}',
    '\u{10ffff}',
];
pub(super) const OPERATORS: [JavaBinaryOperator; 6] = [
    JavaBinaryOperator::Equal,
    JavaBinaryOperator::NotEqual,
    JavaBinaryOperator::Less,
    JavaBinaryOperator::LessEqual,
    JavaBinaryOperator::Greater,
    JavaBinaryOperator::GreaterEqual,
];
pub(super) fn input(name: &str) -> JavaExpr {
    JavaExpr::local(f::int(), f::name(name))
}
pub(super) fn literal(value: char) -> JavaExpr {
    JavaExpr::literal(
        f::int(),
        JavaLiteral::I32(i32::try_from(u32::from(value)).unwrap()),
    )
}
pub(super) fn selection() -> JavaExpr {
    JavaExpr {
        ty: f::int(),
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(JavaExpr::local(f::boolean(), f::name("condition"))),
            when_true: Box::new(input("left")),
            when_false: Box::new(input("right")),
        },
    }
}
pub(super) fn comparison(operator: JavaBinaryOperator) -> JavaExpr {
    JavaExpr {
        ty: f::boolean(),
        precedence: if matches!(
            operator,
            JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual
        ) {
            JavaPrecedence::Equality
        } else {
            JavaPrecedence::Relational
        },
        kind: JavaExprKind::Binary {
            operator,
            left: Box::new(input("left")),
            right: Box::new(input("right")),
        },
    }
}
pub(super) fn function(
    hash: u64,
    name: &str,
    parameters: &[(&str, JavaType)],
    value: JavaExpr,
) -> f::Function {
    f::Function {
        hash,
        public: true,
        name: f::name(name),
        parameters: parameters
            .iter()
            .map(|(name, ty)| JavaParameter {
                ty: ty.clone(),
                name: f::name(name),
                final_parameter: true,
            })
            .collect(),
        result: value.ty.clone(),
        body: JavaBlock::new(vec![JavaStmt::Return(Some(value))]),
    }
}
pub(super) fn functions() -> Vec<f::Function> {
    let mut result: Vec<_> = BOUNDARIES
        .iter()
        .enumerate()
        .map(|(index, value)| {
            function(
                10 + index as u64,
                &format!("literal{index}"),
                &[],
                literal(*value),
            )
        })
        .collect();
    result.push(function(
        40,
        "identity",
        &[("input", f::int())],
        input("input"),
    ));
    let mut local = function(41, "local", &[("input", f::int())], input("value"));
    local.body = JavaBlock::new(vec![
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: f::int(),
            name: f::name("value"),
            value: Some(input("input")),
        },
        JavaStmt::Return(Some(input("value"))),
    ]);
    result.push(local);
    result.push(function(
        42,
        "select",
        &[
            ("condition", f::boolean()),
            ("left", f::int()),
            ("right", f::int()),
        ],
        selection(),
    ));
    result.extend(OPERATORS.into_iter().enumerate().map(|(index, operator)| {
        function(
            50 + index as u64,
            &format!("compare{index}"),
            &[("left", f::int()), ("right", f::int())],
            comparison(operator),
        )
    }));
    result
}
pub(super) fn api(package: TargetAstPackage<JavaDialect>) -> JavaDependencyApi {
    JavaDependencyApi::from_certificate(f::certify(package)).unwrap()
}
pub(super) fn admitted(package: TargetAstPackage<JavaDialect>) -> bool {
    super::super::wrapping_integer::fixture::admitted(package)
}
pub(super) fn imported(
    function: JavaDependencyFunction,
    arguments: Vec<JavaExpr>,
) -> (JavaExpr, JavaDependencyBindings) {
    let result = function.signature().result.clone();
    let (scope, callable) = JavaDependencyScope::new().import(function);
    (
        JavaExpr {
            ty: result,
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Call {
                callable: JavaCallableRef::Dependency(callable),
                receiver: None,
                arguments,
            },
        },
        scope.finish(),
    )
}
pub(super) fn forward(crate_id: u64, original: JavaDependencyFunction) -> JavaDependencyApi {
    let (call, bindings) = imported(original, vec![input("input")]);
    let mut declaration = function(40, "forward", &[("input", f::int())], input("value"));
    declaration.body = JavaBlock::new(vec![
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: f::int(),
            name: f::name("value"),
            value: Some(call),
        },
        JavaStmt::Return(Some(input("value"))),
    ]);
    api(f::package_with_dependencies(
        crate_id,
        vec![declaration],
        bindings,
    ))
}
pub(super) fn chain() -> Vec<JavaDependencyApi> {
    let first = api(f::package(941, functions()));
    let identity = first.function(f::id(941, 40)).unwrap().clone();
    let second = forward(942, identity);
    let third = forward(943, second.functions().next().unwrap().clone());
    vec![first, second, third]
}
