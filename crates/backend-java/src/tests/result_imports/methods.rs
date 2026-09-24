//! Imported operations use only original nominal/member handles.
use super::fixture;
use crate::{ast::*, dialect::*, tests::source_dependency_fixture as source};
use portable_codegen::{RenderReadyPackage, TargetAstPackage};

fn signature(parameters: Vec<JavaType>, result: JavaType) -> JavaMethodSignature {
    JavaMethodSignature {
        receiver: None,
        parameters,
        result,
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    }
}
fn local(ty: JavaType, name: &str) -> JavaExpr {
    JavaExpr::local(ty, source::name(name))
}
fn returned(value: JavaExpr) -> JavaStmt {
    JavaStmt::Return(Some(value))
}
fn function(
    hash: u64,
    name: &str,
    result: JavaType,
    parameters: Vec<JavaType>,
    body: Vec<JavaStmt>,
) -> source::Function {
    source::Function {
        hash,
        public: true,
        name: source::name(name),
        result,
        parameters: parameters
            .into_iter()
            .enumerate()
            .map(|(index, ty)| JavaParameter {
                ty,
                name: source::name(&format!("p{index}")),
                final_parameter: true,
            })
            .collect(),
        body: JavaBlock::new(body),
    }
}
pub(super) fn new(
    constructor: &JavaImportedResultConstructor,
    arguments: Vec<JavaExpr>,
) -> JavaExpr {
    JavaExpr {
        ty: constructor.owner().ty(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::New {
            constructor: JavaConstructorRef::Dependency(constructor.clone()),
            arguments,
        },
    }
}
pub(super) fn upcast(target: JavaType, value: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: target.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target,
            value: Box::new(value),
        },
    }
}
fn nonnull(value: JavaExpr) -> JavaExpr {
    let ty = value.ty.clone();
    JavaExpr {
        ty: ty.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable: JavaKnownCallable::ObjectsRequireNonNull,
                signature: signature(vec![ty.clone()], ty),
            },
            receiver: None,
            arguments: vec![value],
        },
    }
}
fn instance(value: JavaExpr, target: JavaType, binding: &str) -> JavaExpr {
    JavaExpr {
        ty: source::boolean(),
        precedence: JavaPrecedence::Relational,
        kind: JavaExprKind::InstanceOf {
            value: Box::new(value),
            target,
            binding: Some(source::name(binding)),
        },
    }
}
pub(super) fn read(accessor: &JavaImportedResultAccessor, receiver: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: source::int(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: accessor.owner().ty(),
                name: accessor.name().clone(),
                signature: Box::new(accessor.signature()),
                origin: JavaMemberOrigin::Dependency(accessor.clone()),
            },
            receiver: Some(Box::new(receiver)),
            arguments: vec![],
        },
    }
}

pub fn consumer(api: &JavaDependencyApi) -> TargetAstPackage<JavaDialect> {
    let family = api.result_families().next().unwrap();
    let interface = family.ty(JavaResultTypeRole::Interface);
    let success = family.ty(JavaResultTypeRole::Success);
    let error = family.ty(JavaResultTypeRole::Error);
    let (scope, interface) = JavaDependencyScope::new()
        .import_result_type(interface)
        .unwrap();
    let (scope, success) = scope
        .import_result_constructor(success.constructor().unwrap())
        .unwrap();
    let (scope, error) = scope
        .import_result_constructor(error.constructor().unwrap())
        .unwrap();
    let (scope, accessor) = scope
        .import_result_accessor(
            family
                .ty(JavaResultTypeRole::Success)
                .payload_accessor()
                .unwrap(),
        )
        .unwrap();
    let result = interface.ty();
    let payload = success.owner().ty();
    let functions = vec![
        function(
            10,
            "success",
            result.clone(),
            vec![source::int()],
            vec![returned(upcast(
                result.clone(),
                new(&success, vec![local(source::int(), "p0")]),
            ))],
        ),
        function(
            11,
            "error",
            result.clone(),
            vec![],
            vec![returned(upcast(result.clone(), new(&error, vec![])))],
        ),
        function(
            12,
            "copy",
            result.clone(),
            vec![result.clone()],
            vec![returned(nonnull(local(result.clone(), "p0")))],
        ),
        function(
            13,
            "observe",
            source::int(),
            vec![result.clone(), source::int()],
            vec![
                JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: result.clone(),
                    name: source::name("selected"),
                    value: Some(nonnull(local(result.clone(), "p0"))),
                },
                JavaStmt::If {
                    condition: instance(
                        local(result.clone(), "selected"),
                        payload.clone(),
                        "payload",
                    ),
                    then_block: JavaBlock::new(vec![returned(read(
                        &accessor,
                        local(payload.clone(), "payload"),
                    ))]),
                    else_block: None,
                },
                returned(local(source::int(), "p1")),
            ],
        ),
        function(
            14,
            "directProbe",
            source::int(),
            vec![source::int()],
            vec![
                JavaStmt::If {
                    condition: instance(
                        upcast(result, new(&success, vec![local(source::int(), "p0")])),
                        payload.clone(),
                        "direct",
                    ),
                    then_block: JavaBlock::new(vec![returned(read(
                        &accessor,
                        local(payload, "direct"),
                    ))]),
                    else_block: None,
                },
                returned(JavaExpr::literal(source::int(), JavaLiteral::I32(0))),
            ],
        ),
    ];
    source::package_with_dependencies(9, functions, scope.finish())
}
pub fn certificates() -> (JavaDependencyApi, RenderReadyPackage<JavaDialect>) {
    let owner = fixture::owner();
    let consumer = source::certify(consumer(&owner));
    JavaDependencyApi::from_certificate(consumer.clone()).unwrap();
    (owner, consumer)
}

pub fn relay(api: &JavaDependencyApi, crate_id: u64) -> JavaDependencyApi {
    let mut scope = JavaDependencyScope::new();
    let mut functions = vec![];
    for function in api.functions() {
        let (next, imported) = scope.import(function.clone()).unwrap();
        scope = next;
        let signature = imported.signature().clone();
        let arguments = signature
            .parameters
            .iter()
            .enumerate()
            .map(|(index, ty)| {
                let value = local(ty.clone(), &format!("p{index}"));
                if matches!(ty, JavaType::Reference(JavaTypeName::Imported(_))) {
                    nonnull(value)
                } else {
                    value
                }
            })
            .collect();
        functions.push(self::function(
            function.declaration().definition_path_hash,
            function.path().member().as_str(),
            signature.result.clone(),
            signature.parameters,
            vec![returned(JavaExpr {
                ty: signature.result,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Call {
                    callable: JavaCallableRef::Dependency(imported),
                    receiver: None,
                    arguments,
                },
            })],
        ));
    }
    JavaDependencyApi::from_certificate(source::certify(source::package_with_dependencies(
        crate_id,
        functions,
        scope.finish(),
    )))
    .unwrap()
}
