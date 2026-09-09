//! Paired transfers use the same incoming snapshot and invalidate dead memory.
use super::super::numeric::Resolver;
use super::{E, Engine, State};
use crate::ast::contextual::flow_graph::{Destination, Edge, EdgeMeaning, Graph, Point, Polarity};
use crate::ownership::loops::LoopEvidence;
use crate::ownership::{context_facts::ContextFacts, numeric_flow};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Product<'ast> {
    pub(super) memory: State,
    pub(super) numeric: numeric_flow::State<'ast>,
}
impl<'ast> Product<'ast> {
    pub(super) fn action(
        &mut self,
        context: &ContextFacts<'ast>,
        graph: &Graph<'ast>,
        point: Point,
        loops: &[LoopEvidence<'ast>],
        strict: bool,
    ) -> Result<(), E> {
        let pending = self.pending_prefixes(context, graph, point, loops)?;
        let mut engine = Engine {
            context,
            site: Some((graph, point)),
            numeric: Some(self.numeric.clone()),
        };
        let owner_plan = engine.owner_action(graph.node(point).action(), &self.memory);
        let resolver = Resolver {
            engine: &engine,
            memory: &self.memory,
        };
        let mut checker = numeric_flow::Engine::composed(context, &resolver, !strict)?;
        let numeric = if strict {
            checker.verify_action(graph, point, &mut self.numeric)
        } else {
            checker.action(graph.node(point).action(), &mut self.numeric)
        };
        // A failing numeric action is not a successful memory proof either.
        let storage = engine.action(graph.node(point).action(), &mut self.memory);
        let result = numeric
            .and(storage)
            .and(owner_plan)
            .and_then(|plan| plan.apply(&mut self.memory, context.registry()));
        if let Err(error) = result {
            if strict {
                return Err(error);
            }
            self.memory.forget_values();
            self.memory.owners.fail();
            self.numeric.forget(self.memory.roots.keys().cloned());
        } else {
            pending.apply(&mut self.memory);
        }
        self.numeric
            .retain_memory(&self.memory.roots.keys().cloned().collect());
        Ok(())
    }

    pub(super) fn edge(
        &self,
        context: &ContextFacts<'ast>,
        graph: &Graph<'ast>,
        point: Point,
        edge: &Edge<'ast>,
        loops: &[LoopEvidence<'ast>],
        strict: bool,
    ) -> Result<Option<Self>, E> {
        let engine = Engine {
            context,
            site: Some((graph, point)),
            numeric: Some(self.numeric.clone()),
        };
        let resolver = Resolver {
            engine: &engine,
            memory: &self.memory,
        };
        let mut checker = numeric_flow::Engine::composed(context, &resolver, !strict)?;
        let refined = checker.edge(&self.numeric, graph.node(point), edge);
        let numeric = match refined {
            Ok(Some(state)) => state,
            Ok(None) => return Ok(None),
            Err(error) if strict => return Err(error),
            Err(_) => self.numeric.clone(),
        };
        let mut outgoing = Self {
            memory: self.memory.clone(),
            numeric,
        };
        if let EdgeMeaning::Predicate {
            condition,
            polarity,
            ..
        } = edge.meaning()
        {
            let truth = polarity == Polarity::True;
            match engine.branch(condition, truth, &self.memory) {
                Ok(None) => return Ok(None),
                Err(error) if strict => return Err(error),
                _ => {}
            }
            match engine.refine_allocations(condition, truth, &mut outgoing.memory) {
                Ok(false) => return Ok(None),
                Err(error) if strict => return Err(error),
                _ => {}
            }
        }
        outgoing.complete_prefixes(edge, loops);
        if let Err(error) = outgoing.memory.owners.edge(graph, point, edge) {
            if strict {
                return Err(error);
            }
            outgoing.memory.owners.fail();
        }
        for scope in edge.exited_scopes() {
            if let Err(error) = outgoing.memory.owners.leave(scope) {
                if strict {
                    return Err(error);
                }
                outgoing.memory.owners.fail();
            }
            outgoing.memory.leave(scope);
            outgoing.numeric.leave(scope);
        }
        outgoing
            .numeric
            .retain_memory(&outgoing.memory.roots.keys().cloned().collect());
        if let Destination::Point(next) = edge.destination() {
            outgoing.seed_prefixes(graph, next, loops);
        }
        Ok(Some(outgoing))
    }
}
