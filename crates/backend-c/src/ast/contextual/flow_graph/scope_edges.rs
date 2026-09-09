//! Every transfer derives the lexical scopes it crosses, including skipped exits.
use super::{Action, Destination, EdgeMeaning, Graph};
use crate::ast::{CContextError as E, CScopeRef};

pub(super) fn derive(graph: &mut Graph<'_>) -> Result<(), E> {
    let mut inventories = Vec::new();
    for node in &graph.nodes {
        if node.scope.function() != graph.function {
            return Err(E::WrongLexicalOwner);
        }
        let mut exits = Vec::new();
        for edge in &node.successors {
            if matches!(edge.meaning(), EdgeMeaning::Return)
                != matches!(edge.destination(), Destination::FunctionReturn)
            {
                return Err(E::WrongControlTarget);
            }
            let destination = match edge.destination {
                Destination::Point(point) => Some(graph.nodes[point.0].scope),
                Destination::FunctionReturn => None,
            };
            let leaving = match node.action {
                Action::ScopeExit(scope) => {
                    if scope != node.scope {
                        return Err(E::WrongLexicalOwner);
                    }
                    true
                }
                _ => false,
            };
            exits.push(crossed(node.scope, destination, leaving));
        }
        inventories.push(exits);
    }
    for (node, exits) in graph.nodes.iter_mut().zip(inventories) {
        for (edge, scopes) in node.successors.iter_mut().zip(exits) {
            edge.exited_scopes = scopes;
        }
    }
    Ok(())
}

fn crossed<'a>(
    source: &'a CScopeRef,
    destination: Option<&CScopeRef>,
    leaving: bool,
) -> Vec<&'a CScopeRef> {
    let mut crossed = Vec::new();
    let mut scope = Some(source);
    while let Some(current) = scope {
        if !(leaving && current == source) && destination.is_some_and(|to| ancestor(current, to)) {
            break;
        }
        crossed.push(current);
        scope = current.parent();
    }
    crossed
}

fn ancestor(outer: &CScopeRef, inner: &CScopeRef) -> bool {
    let mut scope = Some(inner);
    while let Some(current) = scope {
        if current == outer {
            return true;
        }
        scope = current.parent();
    }
    false
}
