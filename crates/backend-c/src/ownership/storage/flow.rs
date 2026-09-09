//! One numeric/memory fixed point followed by strict replay, never staged guesses.
mod audit;
#[cfg(test)]
#[path = "../../tests/buffer_activation_boundaries.rs"]
mod buffer_tests;
#[cfg(test)]
#[path = "../../tests/buffer_prefix_activations.rs"]
mod prefix_activation_tests;
mod prefixes;
mod product;
mod solve;
use super::root_cells::RootCell;
use super::{Engine, state::State, values::Cell};
use crate::ast::{CDefinitionKind, CFileItem};
use crate::ownership::{CSafetyError as E, context_facts::ContextFacts, loops, paths::Root};

pub(super) fn check<'ast>(context: &ContextFacts<'ast>) -> Result<(), E> {
    // Requested child roles are obligations, never evidence. The finite graph
    // stage must replace this rejection with actual child transition analysis.
    if context.registry().member_ownerships().next().is_some() {
        return Err(E::UnprovedOwnership);
    }
    let loops = loops::check(context)?;
    let mut globals = State::default();
    for file in context.files() {
        for item in file.items() {
            if let CFileItem::Definition(definition) = item
                && let CDefinitionKind::Object { object, .. } = definition.kind()
            {
                globals.roots.insert(
                    Root::Global(object.clone()),
                    RootCell::object(Cell::Uninitialized),
                );
            }
        }
    }
    let mut static_engine = Engine {
        context,
        site: None,
        numeric: None,
    };
    for file in context.files() {
        for item in file.items() {
            if let CFileItem::Definition(definition) = item
                && let CDefinitionKind::Object {
                    object,
                    initializer,
                    ..
                } = definition.kind()
            {
                let cell = static_engine.initializer(initializer, &globals)?;
                cell.complete(object.ty(), context.registry())?;
                globals
                    .roots
                    .insert(Root::Global(object.clone()), RootCell::object(cell));
            }
        }
    }
    // Arbitrary function entry is not program startup.
    for cell in globals.roots.values_mut() {
        *cell = RootCell::object(Cell::Initialized);
    }
    for graph in context.functions() {
        let mut entry = globals.clone();
        for file in context.files() {
            for item in file.items() {
                if let CFileItem::Definition(definition) = item
                    && let CDefinitionKind::Function {
                        function,
                        parameters,
                        ..
                    } = definition.kind()
                    && function == graph.function()
                {
                    for parameter in parameters {
                        entry.roots.insert(
                            Root::Parameter(parameter.clone()),
                            RootCell::object(Cell::Initialized),
                        );
                    }
                }
            }
        }
        let incoming = solve::function(context, graph, entry, &loops)?;
        audit::check(context, graph, &incoming, &loops)?;
    }
    Ok(())
}
