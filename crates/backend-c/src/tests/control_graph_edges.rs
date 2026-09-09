//! Inspect typed edge meaning without relying on vector order.
use super::*;
use crate::ast::contextual_reconstruction::{fixture, int, key, package};
use crate::ast::*;

pub(super) fn body(source: &CSourceFile) -> &CBlock {
    let CFileItem::Definition(value) = &source.items()[0] else {
        panic!("definition")
    };
    let CDefinitionKind::Function { body, .. } = value.kind() else {
        panic!("function")
    };
    body
}
pub(super) fn point(edge: &Edge<'_>) -> Point {
    let Destination::Point(point) = edge.destination() else {
        panic!("point destination")
    };
    point
}

#[test]
fn if_predicate_polarity_remains_attached_to_actual_target_after_edge_reordering() {
    let (mut registry, file, function, scope) = fixture();
    let yes = registry
        .register_scope(&function, Some(&scope), key("yes"))
        .unwrap();
    let no = registry
        .register_scope(&function, Some(&scope), key("no"))
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let condition = values.literal(CLiteral::Bool(true)).unwrap();
    let statement = ast
        .if_statement(
            condition.clone(),
            ast.block(yes.clone(), vec![]).unwrap(),
            ast.block(no.clone(), vec![]).unwrap(),
        )
        .unwrap();
    let source = package(&registry, file, function.clone(), scope, vec![statement]);
    registry
        .check_sequencing_and_control(std::slice::from_ref(&source))
        .unwrap();
    let mut graph = Graph::build(body(&source)).unwrap();
    graph.nodes[graph.entry.0].successors.reverse();
    assert_eq!(graph.function(), &function);
    let branch = graph.node(graph.entry());
    assert_eq!(branch.successors().len(), 2);
    let mut polarities = Vec::new();
    for edge in branch.successors() {
        let EdgeMeaning::Predicate {
            condition: actual,
            polarity,
            owner: BranchOwner::If,
        } = edge.meaning()
        else {
            panic!("If predicate")
        };
        assert_eq!(actual, &condition);
        let Action::Read(source_condition) = branch.action() else {
            panic!("predicate action")
        };
        assert!(std::ptr::eq(actual, *source_condition));
        assert_eq!(
            graph.node(point(edge)).scope(),
            if polarity == Polarity::True {
                &yes
            } else {
                &no
            }
        );
        assert!(edge.exited_scopes().is_empty());
        polarities.push(polarity);
    }
    assert!(polarities.contains(&Polarity::True));
    assert!(polarities.contains(&Polarity::False));
}

#[test]
fn switch_edges_retain_case_lists_discriminant_and_exact_switch_identity() {
    let (mut registry, file, function, scope) = fixture();
    let selected = registry
        .register_scope(&function, Some(&scope), key("selected"))
        .unwrap();
    let fallback = registry
        .register_scope(&function, Some(&scope), key("fallback"))
        .unwrap();
    let identity = registry.register_switch(&scope, key("choice")).unwrap();
    let cases = vec![
        CCaseConstant::Signed(CSignedLiteral::Int(2)),
        CCaseConstant::Signed(CSignedLiteral::Int(5)),
    ];
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let exit = ast
        .break_statement(CBreakTarget::Switch(identity.clone()))
        .unwrap();
    let statement = ast
        .switch_statement(
            identity.clone(),
            int(&values),
            vec![
                ast.switch_arm(
                    cases.clone(),
                    ast.block(selected.clone(), vec![exit.clone()]).unwrap(),
                )
                .unwrap(),
            ],
            ast.block(fallback.clone(), vec![exit]).unwrap(),
        )
        .unwrap();
    let source = package(&registry, file, function, scope, vec![statement]);
    registry
        .check_sequencing_and_control(std::slice::from_ref(&source))
        .unwrap();
    let mut graph = Graph::build(body(&source)).unwrap();
    graph.nodes[graph.entry.0].successors.reverse();
    let mut seen = (false, false);
    for edge in graph.node(graph.entry()).successors() {
        let EdgeMeaning::Switch {
            identity: actual,
            value,
            selection,
        } = edge.meaning()
        else {
            panic!("switch edge")
        };
        assert_eq!(actual, &identity);
        let Action::Read(discriminant) = graph.node(graph.entry()).action() else {
            panic!("switch action")
        };
        assert!(std::ptr::eq(value, *discriminant));
        match selection {
            Selection::Cases(actual_cases) => {
                assert_eq!(actual_cases, cases);
                assert_eq!(graph.node(point(edge)).scope(), &selected);
                seen.0 = true;
            }
            Selection::Default => {
                assert_eq!(graph.node(point(edge)).scope(), &fallback);
                seen.1 = true;
            }
        }
        assert!(edge.exited_scopes().is_empty());
        let target = graph.node(point(edge));
        let exit = &target.successors()[0];
        assert_eq!(
            exit.meaning(),
            EdgeMeaning::Break(&CBreakTarget::Switch(identity.clone()))
        );
        assert_eq!(exit.exited_scopes(), &[target.scope()]);
    }
    assert_eq!(seen, (true, true));
}
