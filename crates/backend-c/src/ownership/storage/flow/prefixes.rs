//! Actual loop transitions carry inductive memory coverage through the product.
use super::product::Product;
use crate::ast::{
    CConstness, CInitializerKind, CObjectTypeKind, CPlaceKind, CScalarType, CValueKind,
    contextual::flow_graph::{Action, BranchOwner, Edge, EdgeMeaning, Graph, Point, Polarity},
};
use crate::ownership::{
    CSafetyError as E,
    context_facts::ContextFacts,
    loops::LoopEvidence,
    paths::{ElementIndex, Key, Root, Shape},
    storage::{
        prefixes::{Bound, join_values},
        state::State,
        values::Cell,
    },
};

pub(super) struct Pending(Vec<(Root, Bound, Option<Cell>)>);
impl Pending {
    pub(super) fn apply(self, memory: &mut State) {
        for (root, bound, mut value) in self.0 {
            // Capturing before the action must not undo that action's normal
            // saved-address invalidation or declaration-activation retirement.
            if let Some(local) = bound.local()
                && let Some(cell) = &mut value
            {
                let changed = Root::Local(local.clone());
                cell.invalidate_index(&changed);
                if matches!(&bound, Bound::Snapshot(_)) {
                    cell.expire(&changed);
                }
            }
            if memory.roots.contains_key(&root) {
                memory
                    .prefixes
                    .entry(root)
                    .or_default()
                    .insert(bound, value);
            }
        }
    }
}

fn counter(evidence: &LoopEvidence<'_>) -> Bound {
    Bound::Counter {
        identity: Box::new(evidence.identity().clone()),
        local: Box::new(evidence.counter().clone()),
    }
}

impl Product<'_> {
    pub(super) fn seed_prefixes(
        &mut self,
        graph: &Graph<'_>,
        point: Point,
        loops: &[LoopEvidence<'_>],
    ) {
        for evidence in loops
            .iter()
            .filter(|evidence| evidence.header(graph, point))
        {
            if self
                .numeric
                .number(&Key::local(evidence.counter()), CScalarType::Size)
                .and_then(|number| number.extent_bounds())
                != Ok((0, 0))
            {
                continue;
            }
            for root in self.memory.roots.keys() {
                if self
                    .memory
                    .allocations
                    .current_count(root)
                    .is_some_and(|count| count.local() == evidence.bound())
                {
                    let prefixes = self.memory.prefixes.entry(root.clone()).or_default();
                    let bound = counter(evidence);
                    if prefixes.get(&bound).is_none() {
                        prefixes.insert(bound, None);
                    }
                }
            }
        }
    }

    pub(super) fn pending_prefixes(
        &self,
        context: &ContextFacts<'_>,
        graph: &Graph<'_>,
        point: Point,
        loops: &[LoopEvidence<'_>],
    ) -> Result<Pending, E> {
        let mut pending = Pending(Vec::new());
        for evidence in loops.iter().filter(|evidence| evidence.step(graph, point)) {
            let source = Key::local(evidence.counter());
            let Ok((first, last)) = self
                .numeric
                .number(&source, CScalarType::Size)
                .and_then(|number| number.extent_bounds())
            else {
                continue;
            };
            let bound = counter(evidence);
            for (root, prefixes) in &self.memory.prefixes {
                let Shape::Elements { element, .. } = root.shape() else {
                    continue;
                };
                if !self
                    .memory
                    .allocations
                    .current_count(root)
                    .is_some_and(|count| count.local() == evidence.bound())
                {
                    continue;
                }
                let Some(old) = prefixes.get(&bound) else {
                    continue;
                };
                let index = ElementIndex::checked(first, last, Some(&source))?;
                let path = Key::from_root(root.clone())
                    .element_selection(index)
                    .ok_or(E::UnprovedStorage)?;
                // Missing complete writes remove the claim on the ordinary step;
                // they are not errors until a later operation requires coverage.
                let Ok(cell) = self.memory.read(&path, context.registry()) else {
                    continue;
                };
                pending.0.push((
                    root.clone(),
                    bound.clone(),
                    join_values(old, &Some(cell), &element, context.registry())?,
                ));
            }
        }
        if let Action::Declare(declaration) = graph.node(point).action() {
            let local = declaration.local();
            let ty = local.ty().canonical();
            if ty.constness() != CConstness::Const
                || !matches!(
                    ty.kind(),
                    CObjectTypeKind::Scalar(CScalarType::Size | CScalarType::U64)
                )
            {
                return Ok(pending);
            }
            let Some(initializer) = declaration.initializer() else {
                return Ok(pending);
            };
            let CInitializerKind::Expression(value) = initializer.kind() else {
                return Ok(pending);
            };
            let CValueKind::Read(place) = value.kind() else {
                return Ok(pending);
            };
            let CPlaceKind::Local(source) = place.kind() else {
                return Ok(pending);
            };
            for (root, prefixes) in &self.memory.prefixes {
                let Shape::Elements { element, .. } = root.shape() else {
                    continue;
                };
                let mut found: Option<Option<Cell>> = None;
                for (_, value) in prefixes
                    .iter()
                    .filter(|(bound, _)| matches!(bound, Bound::Counter { local, .. } if local.as_ref() == source))
                {
                    found = Some(match found {
                        None => value.clone(),
                        Some(old) => join_values(&old, value, &element, context.registry())?,
                    });
                }
                if let Some(value) = found {
                    pending.0.push((
                        root.clone(),
                        Bound::Snapshot(Box::new(local.clone())),
                        value,
                    ));
                }
            }
        }
        Ok(pending)
    }

    pub(super) fn complete_prefixes(&mut self, edge: &Edge<'_>, loops: &[LoopEvidence<'_>]) {
        let EdgeMeaning::Predicate {
            owner: BranchOwner::Loop(identity),
            polarity: Polarity::False,
            ..
        } = edge.meaning()
        else {
            return;
        };
        let Some(evidence) = loops
            .iter()
            .find(|evidence| evidence.identity() == identity)
        else {
            return;
        };
        let bound = counter(evidence);
        for (root, prefixes) in &mut self.memory.prefixes {
            if self
                .memory
                .allocations
                .current_count(root)
                .is_some_and(|count| count.local() == evidence.bound())
                && let Some(value) = prefixes.get(&bound).cloned()
            {
                prefixes.insert(Bound::OriginalCount, value);
            }
        }
    }
}
