//! All actual graph nodes are checked, not just reachable paths.
use super::{CSafetyError as E, context_facts::ContextFacts};
use crate::ast::{
    CDefinitionKind, CFileItem, CInitializerKind, CPlace, CPlaceKind, CValue, CValueKind,
    contextual::{
        access_walk::{self as walk, Access, Visitor},
        flow_graph::Action,
    },
};

struct CallFree;
impl Visitor for CallFree {
    type Error = E;
    fn place(&mut self, _: &CPlace, _: Access) -> Result<(), E> {
        Ok(())
    }
    fn value(&mut self, value: &CValue) -> Result<(), E> {
        if matches!(value.kind(), CValueKind::Call(_)) {
            Err(E::UnsequencedCall)
        } else {
            Ok(())
        }
    }
}

pub(super) fn check(facts: &ContextFacts<'_>) -> Result<(), E> {
    for file in facts.files() {
        for item in file.items() {
            match item {
                CFileItem::Definition(value) => match value.kind() {
                    CDefinitionKind::Object { initializer, .. } => {
                        walk::initializer(&mut CallFree, initializer)?
                    }
                    CDefinitionKind::Function { .. } => {}
                },
                CFileItem::StaticAssert(value) => {
                    walk::expression(&mut CallFree, value.condition())?
                }
                CFileItem::Declaration(_) | CFileItem::Comment(_) => {}
            }
        }
    }
    for graph in facts.functions() {
        for node in graph.nodes() {
            action(node.action())?;
        }
    }
    Ok(())
}

fn root(value: &CValue) -> Result<(), E> {
    match value.kind() {
        CValueKind::Call(value) => walk::call(&mut CallFree, value),
        _ => walk::expression(&mut CallFree, value),
    }
}

fn action(value: &Action<'_>) -> Result<(), E> {
    match value {
        Action::Declare(value) => {
            if let Some(initializer) = value.initializer() {
                match initializer.kind() {
                    CInitializerKind::Expression(value) => root(value)?,
                    _ => walk::initializer(&mut CallFree, initializer)?,
                }
            }
        }
        Action::Assign(place, value) => {
            walk::place(&mut CallFree, place, Access::Write)?;
            if matches!(place.kind(), CPlaceKind::Local(_)) {
                root(value)?;
            } else {
                walk::expression(&mut CallFree, value)?;
            }
        }
        Action::Evaluate(value) => walk::call(&mut CallFree, value.call())?,
        Action::Discard(value) => root(value)?,
        Action::Read(value) => walk::expression(&mut CallFree, value)?,
        Action::Return(value) => {
            if let Some(value) = value {
                root(value)?;
            }
        }
        Action::ScopeExit(_)
        | Action::CleanupJump(_)
        | Action::Label(_)
        | Action::FunctionEnd
        | Action::CaseEnd
        | Action::Empty => {}
    }
    Ok(())
}
