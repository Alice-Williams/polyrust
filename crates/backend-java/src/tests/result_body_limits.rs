//! Exact body work/depth accounting for authenticated imported Result operations.
use super::*;
use crate::{
    ast::{JavaConstructorRef, JavaLiteral, JavaPrecedence},
    dialect::{JavaDependencyApi, JavaDependencyScope, JavaResultTypeRole},
    tests::{result_imports::fixture, source_dependency_fixture as source},
};

#[test]
fn imported_result_constructor_visits_and_depth_have_exact_boundaries() {
    let api = fixture::owner();
    let family = api.result_families().next().unwrap();
    let (scope, constructor) = JavaDependencyScope::new()
        .import_result_constructor(
            family
                .ty(JavaResultTypeRole::Success)
                .constructor()
                .unwrap(),
        )
        .unwrap();
    let expression = JavaExpr {
        ty: constructor.owner().ty(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::New {
            constructor: JavaConstructorRef::Dependency(constructor),
            arguments: vec![JavaExpr::literal(source::int(), JavaLiteral::I32(0))],
        },
    };
    JavaDependencyApi::from_certificate(source::certify(fixture::consumer(
        scope.finish(),
        expression.ty.clone(),
        vec![],
        expression.clone(),
    )))
    .unwrap();
    let mut budget = Budget::new();
    let mut reader = Reader {
        methods: &BTreeMap::new(),
        records: &BTreeMap::new(),
        constants: &BTreeMap::new(),
        budget: &mut budget,
        calls: BTreeSet::new(),
        imported_height: 0,
        mutable_bools: BTreeSet::new(),
        results: &Default::default(),
        nonnull_results: BTreeSet::new(),
    };
    // The authenticated constructor and its payload each consume one visit.
    for _ in 0..MAX_STEPS / 2 {
        reader.expression(&expression, 0).unwrap();
    }
    assert_eq!(reader.budget.remaining, 0);
    assert!(
        reader
            .expression(&expression, 0)
            .unwrap_err()
            .contains("visit limit")
    );
    reader.budget.remaining = 2;
    reader.expression(&expression, MAX_DEPTH - 1).unwrap();
    assert_eq!(reader.budget.remaining, 0);
    reader.budget.remaining = 1;
    assert!(
        reader
            .expression(&expression, 0)
            .unwrap_err()
            .contains("visit limit")
    );
    reader.budget.remaining = 2;
    assert!(
        reader
            .expression(&expression, MAX_DEPTH)
            .unwrap_err()
            .contains("depth limit")
    );
    assert_eq!((MAX_STEPS, MAX_DEPTH), (100_000, 128));
}
