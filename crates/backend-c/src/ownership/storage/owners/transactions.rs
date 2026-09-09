//! Adjacent copy/reset and release/reset transactions cannot cross control choices.
use super::super::{Engine, state::State, values::Pointer};
use super::{Owners, Plan, Slot, Transaction, actions::direct_read};
use crate::ast::contextual::flow_graph::{Action, Destination, Edge, EdgeMeaning, Graph, Point};
use crate::ast::{CCall, CCallableKind, CConversion, CLiteral, CPlaceKind, CValueKind};
use crate::dialect::CKnownCall;
use crate::ownership::{CSafetyError as E, paths::Root};

impl Plan {
    pub(super) fn reset(&mut self, action: &Action<'_>) -> Result<(), E> {
        let Some(pending) = self.owners.pending.take() else {
            return Err(E::UnprovedOwnership);
        };
        let source = match &pending {
            Transaction::Move { source, .. } | Transaction::Drop { source } => source.clone(),
        };
        if !matches!(action, Action::Assign(place, value)
            if matches!(place.kind(), CPlaceKind::Local(local) if local == &source)
            && matches!(value.kind(), CValueKind::Literal(CLiteral::NullPointer(_))))
        {
            return Err(E::UnprovedOwnership);
        }
        let state = match pending {
            Transaction::Move {
                root, destination, ..
            } => {
                self.transferred = Some((*root, *destination));
                Slot::Moved
            }
            Transaction::Drop { .. } => Slot::Dropped,
        };
        self.owners.slots.insert(source.clone(), state);
        Ok(())
    }
}

impl Owners {
    pub(in super::super) fn edge(
        &self,
        graph: &Graph<'_>,
        point: Point,
        edge: &Edge<'_>,
    ) -> Result<(), E> {
        if self.poisoned {
            return Err(E::UnprovedOwnership);
        }
        if self.pending.is_none() {
            return Ok(());
        }
        let Destination::Point(next) = edge.destination() else {
            return Err(E::UnprovedOwnership);
        };
        if graph.node(point).successors().len() != 1
            || graph.node(point).scope() != graph.node(next).scope()
            || edge.meaning() != EdgeMeaning::Flow
            || !edge.exited_scopes().is_empty()
            || graph
                .nodes()
                .iter()
                .flat_map(|n| n.successors())
                .filter(|e| e.destination() == Destination::Point(next))
                .count()
                != 1
        {
            return Err(E::UnprovedOwnership);
        }
        Ok(())
    }
}

impl<'ast> Engine<'_, 'ast> {
    pub(super) fn owner_release(
        &mut self,
        owners: &mut Owners,
        call: &'ast CCall,
        memory: &State,
    ) -> Result<(), E> {
        if call.callable().kind() != &CCallableKind::Known(CKnownCall::Release)
            || owners.slots.is_empty()
        {
            return Ok(());
        }
        let argument = &call.arguments()[0];
        let pointer = self.expression(argument, memory)?.pointer()?;
        let direct = match argument.kind() {
            CValueKind::Convert {
                conversion: CConversion::ObjectToVoid(_),
                operand,
            } => direct_read(operand),
            _ => None,
        };
        if let Some(local) = direct
            && let Some(slot) = owners.slots.get(local)
        {
            match (slot, &pointer) {
                (Slot::Live(root), Pointer::Target(path))
                    if path.whole_root() && path.root() == root.as_ref() =>
                {
                    owners.pending = Some(Transaction::Drop {
                        source: local.clone(),
                    });
                    return Ok(());
                }
                (slot, Pointer::Null) if slot.empty() => return Ok(()),
                _ => return Err(E::UnprovedOwnership),
            }
        }
        let origin = match &pointer {
            Pointer::Allocation(origin) => Some(origin.as_ref()),
            Pointer::Target(path) => match path.root() {
                Root::Allocation(origin, _) => Some(origin.as_ref()),
                _ => None,
            },
            _ => None,
        };
        if origin.is_some_and(|origin| {
            owners.slots.values().any(|slot| {
            matches!(slot, Slot::Live(root) if matches!(root.as_ref(), Root::Allocation(owned, _) if owned.as_ref() == origin))
        })
        }) {
            return Err(E::UnprovedOwnership);
        }
        Ok(())
    }
}
