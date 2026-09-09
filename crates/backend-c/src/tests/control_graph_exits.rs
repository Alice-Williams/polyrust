//! Exits must describe scopes skipped by transfers, not execute fictional nodes.
use super::edge_tests::{body, point};
use super::*;
use crate::ast::contextual_reconstruction::{fixture, key, package};
use crate::ast::*;

#[test]
fn normal_sibling_flow_root_fallthrough_and_nested_return_have_exact_exit_inventories() {
    let (mut registry, file, function, scope) = fixture();
    let first = registry
        .register_scope(&function, Some(&scope), key("first"))
        .unwrap();
    let second = registry
        .register_scope(&function, Some(&scope), key("second"))
        .unwrap();
    let inner = registry
        .register_scope(&function, Some(&second), key("inner"))
        .unwrap();
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let source = package(
        &registry,
        file,
        function,
        scope.clone(),
        vec![
            ast.nested_block(ast.block(first.clone(), vec![]).unwrap())
                .unwrap(),
            ast.nested_block(
                ast.block(
                    second.clone(),
                    vec![
                        ast.nested_block(
                            ast.block(inner.clone(), vec![ast.return_statement(None).unwrap()])
                                .unwrap(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            )
            .unwrap(),
        ],
    );
    registry
        .check_sequencing_and_control(std::slice::from_ref(&source))
        .unwrap();
    let graph = Graph::build(body(&source)).unwrap();
    let first_exit = graph
        .nodes()
        .iter()
        .find(|node| matches!(node.action(), Action::ScopeExit(s) if *s == &first))
        .unwrap();
    assert_eq!(first_exit.successors()[0].exited_scopes(), &[&first]);
    assert_eq!(
        graph.node(point(&first_exit.successors()[0])).scope(),
        &inner
    );
    let root_exit = graph
        .nodes()
        .iter()
        .find(|node| matches!(node.action(), Action::ScopeExit(s) if *s == &scope))
        .unwrap();
    assert_eq!(root_exit.successors()[0].exited_scopes(), &[&scope]);
    assert!(matches!(
        graph.node(point(&root_exit.successors()[0])).action(),
        Action::FunctionEnd
    ));
    let returned = graph
        .nodes()
        .iter()
        .find(|node| matches!(node.action(), Action::Return(_)))
        .unwrap();
    assert_eq!(returned.successors().len(), 1);
    let edge = &returned.successors()[0];
    assert_eq!(edge.destination(), Destination::FunctionReturn);
    assert_eq!(edge.meaning(), EdgeMeaning::Return);
    assert_eq!(edge.exited_scopes(), &[&inner, &second, &scope]);
}

#[test]
fn cleanup_jump_skips_scope_exit_nodes_but_retains_ancestor_scopes_and_label_identity() {
    let (mut registry, file, function, scope) = fixture();
    let outer = registry
        .register_scope(&function, Some(&scope), key("outer"))
        .unwrap();
    let inner = registry
        .register_scope(&function, Some(&outer), key("inner"))
        .unwrap();
    let label = registry
        .register_cleanup_exit(&scope, key("cleanup"))
        .unwrap();
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let source = package(
        &registry,
        file,
        function,
        scope.clone(),
        vec![
            ast.nested_block(
                ast.block(
                    outer.clone(),
                    vec![
                        ast.nested_block(
                            ast.block(
                                inner.clone(),
                                vec![ast.cleanup_jump(label.clone()).unwrap()],
                            )
                            .unwrap(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            )
            .unwrap(),
            ast.label(label.clone(), ast.return_statement(None).unwrap())
                .unwrap(),
        ],
    );
    registry
        .check_sequencing_and_control(std::slice::from_ref(&source))
        .unwrap();
    let graph = Graph::build(body(&source)).unwrap();
    let jump = graph
        .nodes()
        .iter()
        .find(|node| matches!(node.action(), Action::CleanupJump(_)))
        .unwrap();
    let edge = &jump.successors()[0];
    assert_eq!(edge.meaning(), EdgeMeaning::Cleanup(&label));
    assert_eq!(edge.exited_scopes(), &[&inner, &outer]);
    assert!(matches!(graph.node(point(edge)).action(), Action::Label(actual) if *actual == &label));
    assert!(
        graph.node(point(edge)).successors()[0]
            .exited_scopes()
            .is_empty()
    );
    let returned = graph
        .nodes()
        .iter()
        .find(|node| matches!(node.action(), Action::Return(_)))
        .unwrap();
    assert_eq!(returned.successors()[0].exited_scopes(), &[&scope]);
}
