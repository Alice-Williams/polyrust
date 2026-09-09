//! Forward must-analysis, checking reads only after the fixed point converges.

use super::super::{
    CDefinitionKind, CFileItem, CIndexBase, CPlace, CPlaceKind, CRegistry, CReturnType, CSourceFile,
};
use super::{
    CContextError as E,
    access_walk::{self as walk, Access, Visitor},
    flow_graph::{Action, Destination, Graph},
    initialized_paths::{Path, State},
};
use std::collections::VecDeque;

#[cfg(test)]
#[path = "../../tests/contextual_transfer.rs"]
mod tests;

pub(super) fn check(registry: &CRegistry, files: &[CSourceFile]) -> Result<(), E> {
    for file in files {
        for item in file.items() {
            if let CFileItem::Definition(value) = item
                && let CDefinitionKind::Function {
                    function,
                    parameters,
                    body,
                    ..
                } = value.kind()
            {
                let graph = Graph::build(body)?;
                let mut entry = State::default();
                for parameter in parameters {
                    entry.mark(Path::parameter(parameter));
                }
                analyze(
                    registry,
                    &graph,
                    entry,
                    matches!(function.signature().return_type(), CReturnType::Void),
                )?;
            }
        }
    }
    Ok(())
}

fn transfer(action: &Action<'_>, state: &mut State) {
    match action {
        Action::Declare(value) => {
            state.kill_local(value.local());
            if value.initializer().is_some() {
                state.mark(Path::local(value.local()));
            }
        }
        Action::Assign(place, _) => {
            if let Some(path) = Path::place(place) {
                state.mark(path);
            }
        }
        _ => {}
    }
}

fn analyze(registry: &CRegistry, graph: &Graph<'_>, entry: State, void: bool) -> Result<(), E> {
    let mut incoming = vec![None; graph.nodes().len()];
    incoming[graph.entry().index()] = Some(entry);
    let mut pending = VecDeque::from([graph.entry()]);
    while let Some(point) = pending.pop_front() {
        let node = graph.node(point);
        let mut state = incoming[point.index()]
            .clone()
            .expect("queued only after a reachable input");
        transfer(node.action(), &mut state);
        for edge in node.successors() {
            let Destination::Point(next) = edge.destination() else {
                continue;
            };
            let mut outgoing = state.clone();
            for scope in edge.exited_scopes() {
                outgoing.leave_scope(scope);
            }
            let joined = match &incoming[next.index()] {
                None => outgoing,
                Some(old) => old.intersect(&outgoing, registry)?,
            };
            if incoming[next.index()].as_ref() != Some(&joined) {
                incoming[next.index()] = Some(joined);
                pending.push_back(next);
            }
        }
    }
    for (node, state) in graph.nodes().iter().zip(incoming) {
        let Some(mut state) = state else { continue };
        // Scope is retained on every actual point for 02D's lifetime/edge
        // checks, including transfers that skip ordinary ScopeExit nodes.
        if node.scope().function() != graph.function() {
            return Err(E::WrongLexicalOwner);
        }
        if node
            .origin()
            .is_some_and(|statement| statement.function() != node.scope().function())
        {
            return Err(E::WrongLexicalOwner);
        }
        if let Action::Declare(value) = node.action() {
            state.kill_local(value.local());
        }
        let mut reader = Reader {
            registry,
            state: &state,
        };
        match node.action() {
            Action::Declare(value) => {
                if let Some(value) = value.initializer() {
                    walk::initializer(&mut reader, value)?;
                }
            }
            Action::Assign(place, value) => {
                walk::expression(&mut reader, value)?;
                walk::place(&mut reader, place, Access::Write)?;
            }
            Action::Evaluate(value) => walk::call(&mut reader, value.call())?,
            Action::Read(value) | Action::Discard(value) => walk::expression(&mut reader, value)?,
            Action::Return(Some(value)) => walk::expression(&mut reader, value)?,
            Action::FunctionEnd if !void => return Err(E::MissingReturn),
            Action::CaseEnd => return Err(E::SwitchFallthrough),
            Action::Label(identity) if identity.scope() != node.scope() => {
                return Err(E::WrongLexicalOwner);
            }
            _ => {}
        }
    }
    Ok(())
}

struct Reader<'a> {
    registry: &'a CRegistry,
    state: &'a State,
}
impl Visitor for Reader<'_> {
    type Error = E;
    fn evaluation(&self) -> walk::Evaluation {
        walk::Evaluation::RuntimePaths
    }
    fn place(&mut self, place: &CPlace, access: Access) -> Result<(), E> {
        if access != Access::Read {
            return Ok(());
        }
        if let Some(path) = Path::place(place) {
            if !self.state.covers(&path, self.registry)? {
                return Err(E::UninitializedRead);
            }
        } else if let CPlaceKind::Member { base, .. } = place.kind() {
            // A member behind a nonconstant local array index still reads
            // local storage. Do not lose that obligation with the exact path.
            self.place(base, Access::Read)?;
        } else if let CPlaceKind::Index {
            base: CIndexBase::Array(base),
            ..
        } = place.kind()
        {
            // A variable index may read any element. Whole-array coverage is
            // sufficient; partial heap/prefix/range proofs belong to 02D.
            self.place(base, Access::Read)?;
        }
        Ok(())
    }
}
