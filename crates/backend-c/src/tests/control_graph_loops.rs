//! Continue ignores intervening switches; Break and backedges keep exact owners.
use super::edge_tests::{body, point};
use super::*;
use crate::ast::contextual_reconstruction::{fixture, int, key, package};
use crate::ast::*;

#[test]
fn loop_predicates_continue_switch_break_loop_break_and_backedge_are_distinct() {
    let (mut registry, file, function, scope) = fixture();
    let loop_scope = registry
        .register_scope(&function, Some(&scope), key("loop_body"))
        .unwrap();
    let arm_scope = registry
        .register_scope(&function, Some(&loop_scope), key("arm"))
        .unwrap();
    let default_scope = registry
        .register_scope(&function, Some(&loop_scope), key("fallback"))
        .unwrap();
    let iteration = registry.register_loop(&scope, key("iteration")).unwrap();
    let selection = registry
        .register_switch(&loop_scope, key("selection"))
        .unwrap();
    let size = CObjectType::scalar(CScalarType::Size);
    let counter = registry
        .register_local(&scope, key("counter"), size.clone())
        .unwrap();
    let bound = registry
        .register_local(
            &scope,
            key("bound"),
            size.with_constness(CConstness::Const).unwrap(),
        )
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let zero = values
        .expression_initializer(
            values
                .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
                .unwrap(),
        )
        .unwrap();
    let switch = ast
        .switch_statement(
            selection.clone(),
            int(&values),
            vec![
                ast.switch_arm(
                    vec![CCaseConstant::Signed(CSignedLiteral::Int(1))],
                    ast.block(
                        arm_scope.clone(),
                        vec![ast.continue_statement(iteration.clone()).unwrap()],
                    )
                    .unwrap(),
                )
                .unwrap(),
            ],
            ast.block(
                default_scope.clone(),
                vec![
                    ast.break_statement(CBreakTarget::Switch(selection.clone()))
                        .unwrap(),
                ],
            )
            .unwrap(),
        )
        .unwrap();
    let loop_body = ast
        .block(
            loop_scope.clone(),
            vec![
                switch,
                ast.break_statement(CBreakTarget::Loop(iteration.clone()))
                    .unwrap(),
            ],
        )
        .unwrap();
    let source = package(
        &registry,
        file,
        function,
        scope.clone(),
        vec![
            ast.declare(counter.clone(), Some(zero.clone())).unwrap(),
            ast.declare(bound.clone(), Some(zero)).unwrap(),
            ast.counted_loop(
                iteration.clone(),
                counter,
                bound,
                values.literal(CLiteral::Bool(false)).unwrap(),
                loop_body,
            )
            .unwrap(),
        ],
    );
    registry
        .check_sequencing_and_control(std::slice::from_ref(&source))
        .unwrap();
    let graph = Graph::build(body(&source)).unwrap();
    let test = graph
        .nodes()
        .iter()
        .find(|node| {
            node.origin()
                .is_some_and(|v| matches!(v.kind(), CStatementKind::BoundedLoop { .. }))
        })
        .unwrap();
    for edge in test.successors() {
        let EdgeMeaning::Predicate {
            condition,
            polarity,
            owner: BranchOwner::Loop(actual),
        } = edge.meaning()
        else {
            panic!("loop predicate")
        };
        assert_eq!(actual, &iteration);
        let Action::Read(predicate) = test.action() else {
            panic!("loop read")
        };
        assert!(std::ptr::eq(condition, *predicate));
        assert_eq!(
            graph.node(point(edge)).scope(),
            if polarity == Polarity::True {
                &loop_scope
            } else {
                &scope
            }
        );
        assert!(edge.exited_scopes().is_empty());
    }
    let mut found = [false; 4];
    for node in graph.nodes() {
        for edge in node.successors() {
            match edge.meaning() {
                EdgeMeaning::Continue(actual) => {
                    assert_eq!(actual, &iteration);
                    assert!(std::ptr::eq(graph.node(point(edge)), test));
                    assert_eq!(edge.exited_scopes(), &[&arm_scope, &loop_scope]);
                    found[0] = true;
                }
                EdgeMeaning::Backedge(actual) => {
                    assert_eq!(actual, &iteration);
                    assert!(std::ptr::eq(graph.node(point(edge)), test));
                    assert_eq!(edge.exited_scopes(), &[&loop_scope]);
                    found[1] = true;
                }
                EdgeMeaning::Break(CBreakTarget::Switch(actual)) => {
                    assert_eq!(actual, &selection);
                    assert_eq!(edge.exited_scopes(), &[&default_scope]);
                    assert!(
                        matches!(graph.node(point(edge)).origin().unwrap().kind(), CStatementKind::Break(CBreakTarget::Loop(actual)) if actual == &iteration)
                    );
                    found[2] = true;
                }
                EdgeMeaning::Break(CBreakTarget::Loop(actual)) => {
                    assert_eq!(actual, &iteration);
                    assert_eq!(edge.exited_scopes(), &[&loop_scope]);
                    assert!(
                        matches!(graph.node(point(edge)).action(), Action::ScopeExit(actual) if *actual == &scope)
                    );
                    found[3] = true;
                }
                _ => {}
            }
        }
    }
    assert_eq!(found, [true; 4]);
}
