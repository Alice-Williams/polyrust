//! Inspect actual loop graph actions with a previous-iteration storage fact.
use super::*;
use crate::ast::contextual_reconstruction::{fixture, int, key};
use crate::ast::*;

#[test]
fn loop_declaration_clears_previous_iteration_state_before_initializer() {
    for initialized in [false, true] {
        let (mut registry, _, function, scope) = fixture();
        let body_scope = registry
            .register_scope(&function, Some(&scope), key("loop_body"))
            .unwrap();
        let local = registry
            .register_local(
                &body_scope,
                key("local"),
                CObjectType::scalar(CScalarType::I32),
            )
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
        let loop_id = registry.register_loop(&scope, key("iteration")).unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let body = ast
            .block(
                body_scope,
                vec![
                    ast.declare(
                        local.clone(),
                        initialized.then(|| values.expression_initializer(int(&values)).unwrap()),
                    )
                    .unwrap(),
                    ast.assign(values.local(local.clone()).unwrap(), int(&values))
                        .unwrap(),
                    ast.continue_statement(loop_id.clone()).unwrap(),
                ],
            )
            .unwrap();
        let iteration = ast
            .counted_loop(
                loop_id,
                counter,
                bound,
                values.literal(CLiteral::Bool(false)).unwrap(),
                body,
            )
            .unwrap();
        let root = ast.block(scope, vec![iteration]).unwrap();
        let graph = Graph::build(&root).unwrap();
        let declaration = graph
            .nodes()
            .iter()
            .find(|node| matches!(node.action(), Action::Declare(value) if value.local() == &local))
            .unwrap();
        let continuation = graph
            .nodes()
            .iter()
            .find(|node| {
                node.origin()
                    .is_some_and(|value| matches!(value.kind(), CStatementKind::Continue(_)))
            })
            .unwrap();
        assert_eq!(continuation.successors().len(), 1);
        let Destination::Point(repeat) = continuation.successors()[0].destination() else {
            panic!("Continue must have a point destination");
        };
        let repeat = graph.node(repeat);
        assert!(
            repeat
                .origin()
                .is_some_and(|value| matches!(value.kind(), CStatementKind::BoundedLoop { .. }))
        );
        let path = Path::local(&local);
        let mut previous_iteration = State::default();
        previous_iteration.mark(path.clone());
        assert!(previous_iteration.covers(&path, &registry).unwrap());
        let mut outgoing = previous_iteration.clone();
        for scope in continuation.successors()[0].exited_scopes() {
            outgoing.leave_scope(scope);
        }
        assert!(!outgoing.covers(&path, &registry).unwrap());
        transfer(declaration.action(), &mut previous_iteration);
        assert_eq!(
            previous_iteration.covers(&path, &registry).unwrap(),
            initialized
        );
        // This is a graph/transfer control, not a whole-program certificate:
        // arithmetic progress and declaration dominance have separate tests.
    }
}
