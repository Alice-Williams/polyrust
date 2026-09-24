//! Typed selected-arm controls; no observer or mutable test state enters the AST.
use super::super::{fixture, methods};
use crate::{ast::*, dialect::*, tests::source_dependency_fixture as source};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mutation {
    None,
    InactiveArm,
    DuplicateScrutinee,
    ArmBeforeScrutinee,
    TagFlip,
    PayloadFlip,
}

pub(crate) struct Packages {
    pub family: JavaDependencyApi,
    pub producer: JavaDependencyApi,
    pub relay: JavaDependencyApi,
}

fn local(ty: JavaType, name: &str) -> JavaExpr {
    JavaExpr::local(ty, source::name(name))
}
fn number(value: i32) -> JavaExpr {
    JavaExpr::literal(source::int(), JavaLiteral::I32(value))
}
fn returned(value: JavaExpr) -> JavaStmt {
    JavaStmt::Return(Some(value))
}
fn bound(name: &str, value: JavaExpr) -> JavaStmt {
    JavaStmt::Local {
        finality: JavaLocalFinality::Final,
        ty: value.ty.clone(),
        name: source::name(name),
        value: Some(value),
    }
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
fn call(function: &JavaImportedCallable, arguments: Vec<JavaExpr>) -> JavaExpr {
    JavaExpr {
        ty: function.signature().result.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Dependency(function.clone()),
            receiver: None,
            arguments,
        },
    }
}
fn lookup(api: &JavaDependencyApi, name: &str) -> JavaDependencyFunction {
    api.functions()
        .find(|function| function.path().member().as_str() == name)
        .unwrap()
        .clone()
}

fn producer(family: &JavaDependencyApi, mutation: Mutation) -> JavaDependencyApi {
    let family = family.result_families().next().unwrap();
    let (scope, result) = JavaDependencyScope::new()
        .import_result_type(family.ty(JavaResultTypeRole::Interface))
        .unwrap();
    let (scope, success) = scope
        .import_result_constructor(
            family
                .ty(JavaResultTypeRole::Success)
                .constructor()
                .unwrap(),
        )
        .unwrap();
    let (scope, error) = scope
        .import_result_constructor(family.ty(JavaResultTypeRole::Error).constructor().unwrap())
        .unwrap();
    let payload = local(source::int(), "p1");
    let payload = if mutation == Mutation::PayloadFlip {
        JavaExpr {
            ty: source::int(),
            precedence: JavaPrecedence::BitXor,
            kind: JavaExprKind::Binary {
                operator: JavaBinaryOperator::BitXor,
                left: Box::new(payload),
                right: Box::new(number(1)),
            },
        }
    } else {
        payload
    };
    let yes = methods::upcast(result.ty(), methods::new(&success, vec![payload]));
    let no = methods::upcast(result.ty(), methods::new(&error, vec![]));
    let (yes, no) = if mutation == Mutation::TagFlip {
        (no, yes)
    } else {
        (yes, no)
    };
    let functions = vec![
        function(
            20,
            "select",
            result.ty(),
            vec![source::boolean(), source::int()],
            vec![
                JavaStmt::If {
                    condition: local(source::boolean(), "p0"),
                    then_block: JavaBlock::new(vec![returned(yes)]),
                    else_block: None,
                },
                returned(no),
            ],
        ),
        function(
            21,
            "successArm",
            source::int(),
            vec![source::int()],
            vec![returned(local(source::int(), "p0"))],
        ),
        function(
            22,
            "errorArm",
            source::int(),
            vec![],
            vec![returned(number(17))],
        ),
    ];
    JavaDependencyApi::from_certificate(source::certify(source::package_with_dependencies(
        9,
        functions,
        scope.finish(),
    )))
    .unwrap()
}

pub(crate) fn packages(mutation: Mutation) -> Packages {
    let family = fixture::owner();
    let producer = producer(&family, mutation);
    let (scope, selector) = JavaDependencyScope::new()
        .import(lookup(&producer, "select"))
        .unwrap();
    let (scope, success) = scope.import(lookup(&producer, "successArm")).unwrap();
    let (scope, error) = scope.import(lookup(&producer, "errorArm")).unwrap();
    let (scope, accessor) = scope
        .import_result_accessor(
            family
                .result_families()
                .next()
                .unwrap()
                .ty(JavaResultTypeRole::Success)
                .payload_accessor()
                .unwrap(),
        )
        .unwrap();
    let select = || {
        call(
            &selector,
            vec![local(source::boolean(), "p0"), local(source::int(), "p1")],
        )
    };
    let result = selector.signature().result.clone();
    let success_call = || call(&success, vec![local(source::int(), "p1")]);
    let error_call = || call(&error, vec![]);
    let body = if mutation == Mutation::ArmBeforeScrutinee {
        // Both arms preserve values, but invoke their helper before the selector.
        let branch = |arm| {
            JavaBlock::new(vec![
                bound("early", arm),
                bound("selected", select()),
                returned(local(source::int(), "early")),
            ])
        };
        vec![JavaStmt::If {
            condition: local(source::boolean(), "p0"),
            then_block: branch(success_call()),
            else_block: Some(branch(error_call())),
        }]
    } else {
        let mut body = vec![];
        if mutation == Mutation::DuplicateScrutinee {
            body.push(bound("duplicate", select()));
        }
        body.push(bound("selected", select()));
        let mut yes = vec![];
        let mut no = vec![];
        if mutation == Mutation::InactiveArm {
            yes.push(bound("inactive", error_call()));
            no.push(bound("inactive", success_call()));
        }
        yes.push(returned(call(
            &success,
            vec![methods::read(
                &accessor,
                local(accessor.owner().ty(), "payload"),
            )],
        )));
        no.push(returned(error_call()));
        body.push(JavaStmt::If {
            condition: JavaExpr {
                ty: source::boolean(),
                precedence: JavaPrecedence::Relational,
                kind: JavaExprKind::InstanceOf {
                    value: Box::new(local(result, "selected")),
                    target: accessor.owner().ty(),
                    binding: Some(source::name("payload")),
                },
            },
            then_block: JavaBlock::new(yes),
            else_block: Some(JavaBlock::new(no)),
        });
        body
    };
    let relay =
        JavaDependencyApi::from_certificate(source::certify(source::package_with_dependencies(
            10,
            vec![function(
                30,
                "entry",
                source::int(),
                vec![source::boolean(), source::int()],
                body,
            )],
            scope.finish(),
        )))
        .unwrap();
    Packages {
        family,
        producer,
        relay,
    }
}
