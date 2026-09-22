//! Actual source-owned target packages, not raw Java body templates.
use super::*;

pub(super) fn wide() -> JavaType {
    JavaType::primitive(JavaPrimitive::Long)
}
pub(super) fn cast(value: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: wide(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: wide(),
            value: Box::new(value),
        },
    }
}
pub(super) fn input() -> JavaExpr {
    JavaExpr::local(f::int(), f::name("input"))
}
pub(super) fn function(value: JavaExpr) -> f::Function {
    f::Function {
        hash: 10,
        public: true,
        name: f::name("widen"),
        parameters: vec![JavaParameter {
            ty: f::int(),
            name: f::name("input"),
            final_parameter: true,
        }],
        result: value.ty.clone(),
        body: JavaBlock::new(vec![JavaStmt::Return(Some(value))]),
    }
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
pub(super) fn chain(materialized: bool) -> Vec<JavaDependencyApi> {
    let identity = api(f::package(811, vec![function(input())]));
    let (call, bindings) = imported(identity.functions().next().unwrap().clone(), vec![input()]);
    let mut declaration = function(cast(call.clone()));
    if materialized {
        declaration.body = JavaBlock::new(vec![
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: f::int(),
                name: f::name("operand"),
                value: Some(call),
            },
            JavaStmt::Return(Some(cast(JavaExpr::local(f::int(), f::name("operand"))))),
        ]);
    }
    let widening = api(f::package_with_dependencies(
        812,
        vec![declaration],
        bindings,
    ));
    let (call, bindings) = imported(widening.functions().next().unwrap().clone(), vec![input()]);
    let forward = api(f::package_with_dependencies(
        813,
        vec![function(call)],
        bindings,
    ));
    vec![identity, widening, forward]
}
