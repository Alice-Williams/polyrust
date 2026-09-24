use super::*;
use crate::ast::JavaLiteral;
use crate::ast::{
    JavaMethodDeclaration, JavaMethodSignature, JavaModifier, JavaPrecedence, JavaVisibility,
};
use crate::tests::source_dependency_fixture::*;
use portable_codegen::{GeneratedCallable, GeneratedOrigin, SynthesisReason, TargetAstBuilder};
use portable_diagnostics::SourceRef;

fn ids(count: usize) -> Vec<GeneratedCallableId> {
    let mut builder = TargetAstBuilder::new(crate::dialect::JavaDialect);
    (0..count)
        .map(|index| {
            builder.callable(GeneratedCallable {
                name: format!("f{index}"),
                signature: crate::dialect::JavaDialect.coarse_signature(&JavaMethodSignature {
                    receiver: None,
                    parameters: vec![],
                    result: int(),
                    checked_exceptions: vec![],
                    nullable_result: false,
                    pure: true,
                }),
                visibility: JavaVisibility::Private,
                origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
                source: SourceRef::logical(["body-limit"]),
            })
        })
        .collect()
}

#[test]
fn mutable_bool_body_admission_restores_lexical_scope_and_rejects_parameter_writes() {
    let methods = BTreeMap::new();
    let records = BTreeMap::new();
    let mut budget = Budget::new();
    let mut reader = Reader {
        results: &Default::default(),
        nonnull_results: BTreeSet::new(),
        methods: &methods,
        records: &records,
        constants: &BTreeMap::new(),
        budget: &mut budget,
        calls: BTreeSet::new(),
        imported_height: 0,
        mutable_bools: BTreeSet::new(),
    };
    let declaration = JavaStmt::Local {
        finality: JavaLocalFinality::Mutable,
        ty: boolean(),
        name: name("p0"),
        value: Some(JavaExpr::literal(boolean(), JavaLiteral::Boolean(false))),
    };
    let assignment = JavaStmt::Assign {
        target: JavaExpr::local(boolean(), name("p0")),
        value: JavaExpr::literal(boolean(), JavaLiteral::Boolean(true)),
    };
    reader
        .block(
            &JavaBlock::new(vec![declaration.clone(), assignment.clone()]),
            0,
        )
        .unwrap();
    assert!(reader.mutable_bools.is_empty());
    // A completed method/block cannot authorize the same spelling in the next.
    assert!(
        reader
            .block(&JavaBlock::new(vec![assignment.clone()]), 0)
            .unwrap_err()
            .contains("unadmitted statement")
    );
    // A declaration in the taken branch cannot authorize a sibling-branch write.
    let sibling = JavaStmt::If {
        condition: JavaExpr::literal(boolean(), JavaLiteral::Boolean(true)),
        then_block: JavaBlock::new(vec![declaration]),
        else_block: Some(JavaBlock::new(vec![assignment])),
    };
    assert!(
        reader
            .block(&JavaBlock::new(vec![sibling]), 0)
            .unwrap_err()
            .contains("unadmitted statement")
    );
}

#[test]
fn call_graph_exact_height_one_over_cycle_and_shared_diamond() {
    let ids = ids(129);
    let chain = |length: usize| {
        (0..length)
            .map(|index| {
                (
                    ids[index],
                    if index == 0 {
                        BTreeSet::new()
                    } else {
                        BTreeSet::from([ids[index - 1]])
                    },
                )
            })
            .collect()
    };
    call_heights(&chain(128)).unwrap();
    assert!(call_heights(&chain(129)).unwrap_err().contains("height"));
    let mut graph = chain(3);
    graph.insert(ids[0], BTreeSet::from([ids[2]]));
    assert!(call_heights(&graph).unwrap_err().contains("recursive"));
    graph.insert(ids[0], BTreeSet::new());
    graph.insert(ids[3], BTreeSet::from([ids[0], ids[1], ids[2]]));
    call_heights(&graph).unwrap();
}

#[test]
fn body_budget_and_depth_charge_exact_and_one_over() {
    let methods = BTreeMap::new();
    let records = BTreeMap::new();
    let mut budget = Budget { remaining: 1 };
    let mut reader = Reader {
        results: &Default::default(),
        nonnull_results: BTreeSet::new(),
        methods: &methods,
        records: &records,
        constants: &BTreeMap::new(),
        budget: &mut budget,
        calls: BTreeSet::new(),
        imported_height: 0,
        mutable_bools: BTreeSet::new(),
    };
    reader.charge(MAX_DEPTH).unwrap();
    assert!(reader.charge(0).unwrap_err().contains("visit"));
    reader.budget.remaining = MAX_STEPS;
    assert!(reader.charge(MAX_DEPTH + 1).unwrap_err().contains("depth"));
    assert_eq!((MAX_FUNCTIONS, MAX_STEPS, MAX_DEPTH), (4096, 100_000, 128));
}

#[test]
fn actual_function_inventory_and_expression_depth_boundaries() {
    let ids = ids(MAX_FUNCTIONS + 1);
    let method = JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Private, JavaModifier::Static],
        type_parameters: vec![],
        return_type: int(),
        name: name("leaf"),
        parameters: vec![],
        body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
            JavaExpr::literal(int(), JavaLiteral::I32(0)),
        ))])),
    };
    let records = BTreeMap::new();
    let methods = ids[..MAX_FUNCTIONS]
        .iter()
        .map(|id| (*id, &method))
        .collect();
    verify(&methods, &records, &BTreeMap::new(), &mut Budget::new()).unwrap();
    let methods = ids.iter().map(|id| (*id, &method)).collect();
    assert!(
        verify(&methods, &records, &BTreeMap::new(), &mut Budget::new())
            .unwrap_err()
            .contains("function limit")
    );
    let nested = |depth| {
        (0..depth).fold(JavaExpr::literal(int(), JavaLiteral::I32(0)), |value, _| {
            JavaExpr {
                ty: int(),
                precedence: JavaPrecedence::Conditional,
                kind: JavaExprKind::Conditional {
                    condition: Box::new(JavaExpr::literal(boolean(), JavaLiteral::Boolean(true))),
                    when_true: Box::new(value),
                    when_false: Box::new(JavaExpr::literal(int(), JavaLiteral::I32(1))),
                },
            }
        })
    };
    let mut budget = Budget::new();
    let mut reader = Reader {
        results: &Default::default(),
        nonnull_results: BTreeSet::new(),
        methods: &methods,
        records: &records,
        constants: &BTreeMap::new(),
        budget: &mut budget,
        calls: BTreeSet::new(),
        imported_height: 0,
        mutable_bools: BTreeSet::new(),
    };
    reader.expression(&nested(MAX_DEPTH), 0).unwrap();
    assert!(
        reader
            .expression(&nested(MAX_DEPTH + 1), 0)
            .unwrap_err()
            .contains("depth limit")
    );
}

#[test]
fn constructors_and_methods_share_one_exact_package_budget() {
    for reverse in [false, true] {
        let package = certify(record_package(|fixture| {
            if reverse {
                fixture.facade.members.reverse();
            }
        }));
        // 1 empty facade block + 9 canonical record nodes + 12 method nodes.
        let mut exact = Budget { remaining: 22 };
        let inventory = super::super::inventory::collect_with_budget(&package, &mut exact).unwrap();
        assert_eq!(inventory.functions.len(), 1);
        assert_eq!(exact.remaining, 0);
        let mut one_over = Budget { remaining: 21 };
        let Err(error) = super::super::inventory::collect_with_budget(&package, &mut one_over)
        else {
            panic!("record constructors and methods must not receive separate budgets")
        };
        assert!(error.contains("body visit limit"), "{error}");
    }
    let package = certify(package(7, functions(42)));
    // Even the no-record case charges the empty facade constructor.
    let mut exact = Budget { remaining: 13 };
    super::super::inventory::collect_with_budget(&package, &mut exact).unwrap();
    assert_eq!(exact.remaining, 0);
    assert!(
        super::super::inventory::collect_with_budget(&package, &mut Budget { remaining: 12 })
            .is_err()
    );
}
