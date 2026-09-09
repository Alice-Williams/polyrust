//! Lexical dominance is independent of reachability and value initialization.

use super::super::{
    CAllocatorSource, CBlock, CBreakTarget, CCleanupExitRef, CConversion, CDefinitionKind,
    CFileItem, CFunctionRef, CLocalRef, CPlace, CPlaceKind, CRegistry, CScopeRef, CSourceFile,
    CStatement, CStatementKind, CValue, CValueKind,
};
use super::{
    CContextError as E,
    access_walk::{self as walk, Access, Visitor},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn check(registry: &CRegistry, files: &[CSourceFile]) -> Result<(), E> {
    for file in files {
        for item in file.items() {
            if let CFileItem::Definition(value) = item
                && let CDefinitionKind::Function { function, body, .. } = value.kind()
            {
                let mut checker = Lexical::new(registry, function);
                checker.block(body)?;
                checker.cleanup()?;
            }
        }
    }
    Ok(())
}

#[derive(Clone)]
struct Location {
    ordinal: usize,
    scope: CScopeRef,
    visible: BTreeSet<CLocalRef>,
}

struct Lexical<'a> {
    registry: &'a CRegistry,
    function: &'a CFunctionRef,
    scopes: Vec<CScopeRef>,
    visible: BTreeSet<CLocalRef>,
    controls: Vec<CBreakTarget>,
    ordinal: usize,
    labels: BTreeMap<CCleanupExitRef, Location>,
    jumps: Vec<(CCleanupExitRef, Location)>,
}

impl<'a> Lexical<'a> {
    fn new(registry: &'a CRegistry, function: &'a CFunctionRef) -> Self {
        Self {
            registry,
            function,
            scopes: Vec::new(),
            visible: BTreeSet::new(),
            controls: Vec::new(),
            ordinal: 0,
            labels: BTreeMap::new(),
            jumps: Vec::new(),
        }
    }

    fn scope(&self) -> &CScopeRef {
        self.scopes.last().expect("walking a function block")
    }

    fn location(&self) -> Location {
        Location {
            ordinal: self.ordinal,
            scope: self.scope().clone(),
            visible: self.visible.clone(),
        }
    }

    fn block(&mut self, block: &CBlock) -> Result<(), E> {
        if block.scope().function() != self.function || block.scope().parent() != self.scopes.last()
        {
            return Err(E::WrongLexicalOwner);
        }
        let outer = self.visible.clone();
        self.scopes.push(block.scope().clone());
        for statement in block.statements() {
            self.statement(statement)?;
        }
        self.scopes.pop();
        self.visible = outer;
        Ok(())
    }

    fn statement(&mut self, statement: &CStatement) -> Result<(), E> {
        self.ordinal = self.ordinal.checked_add(1).ok_or(E::TraversalCapacity)?;
        match statement.kind() {
            CStatementKind::Empty => {}
            CStatementKind::Block(block) => self.block(block)?,
            CStatementKind::Declare(value) => {
                if value.local().scope() != self.scope() {
                    return Err(E::WrongLexicalOwner);
                }
                self.visible.insert(value.local().clone());
                // Declaration is visible in its own initializer; the separate
                // flow analysis must reject reading its uninitialized value.
                if let Some(value) = value.initializer() {
                    walk::initializer(self, value)?;
                }
            }
            CStatementKind::Assign { place, value } => {
                walk::expression(self, value)?;
                walk::place(self, place, Access::Write)?;
            }
            CStatementKind::Evaluate(effect) => walk::call(self, effect.call())?,
            CStatementKind::Discard(value) => walk::expression(self, value)?,
            CStatementKind::If {
                condition,
                then_block,
                else_block,
            } => {
                walk::expression(self, condition)?;
                self.block(then_block)?;
                self.block(else_block)?;
            }
            CStatementKind::BoundedLoop {
                identity,
                progress,
                condition,
                body,
            } => {
                if identity.scope() != self.scope() {
                    return Err(E::WrongLexicalOwner);
                }
                self.local(progress.counter())?;
                self.local(progress.bound())?;
                walk::expression(self, condition)?;
                self.controls.push(CBreakTarget::Loop(identity.clone()));
                self.block(body)?;
                self.controls.pop();
            }
            CStatementKind::Switch {
                identity,
                value,
                arms,
                default,
            } => {
                super::case_constants::check(self.registry, value, arms)?;
                if identity.scope() != self.scope() {
                    return Err(E::WrongLexicalOwner);
                }
                walk::expression(self, value)?;
                self.controls.push(CBreakTarget::Switch(identity.clone()));
                for arm in arms {
                    self.block(arm.body())?;
                }
                self.block(default)?;
                self.controls.pop();
            }
            CStatementKind::Break(target) => {
                if self.controls.last() != Some(target) {
                    return Err(E::WrongControlTarget);
                }
            }
            CStatementKind::Continue(target) => {
                let actual = self.controls.iter().rev().find_map(|value| match value {
                    CBreakTarget::Loop(value) => Some(value),
                    CBreakTarget::Switch(_) => None,
                });
                if actual != Some(target) {
                    return Err(E::WrongControlTarget);
                }
            }
            CStatementKind::Return(value) => {
                if let Some(value) = value {
                    walk::expression(self, value)?;
                }
            }
            CStatementKind::CleanupJump(identity) => {
                self.jumps.push((identity.clone(), self.location()));
            }
            CStatementKind::Label {
                identity,
                statement,
            } => {
                if identity.scope() != self.scope() {
                    return Err(E::WrongLexicalOwner);
                }
                if self
                    .labels
                    .insert(identity.clone(), self.location())
                    .is_some()
                {
                    return Err(E::DuplicateOccurrence);
                }
                self.statement(statement)?;
            }
        }
        Ok(())
    }

    fn local(&self, value: &CLocalRef) -> Result<(), E> {
        if value.scope().function() != self.function {
            return Err(E::WrongLexicalOwner);
        }
        if !self.visible.contains(value) {
            return Err(E::InvisibleBinding);
        }
        Ok(())
    }

    fn cleanup(&self) -> Result<(), E> {
        for (identity, from) in &self.jumps {
            let to = self.labels.get(identity).ok_or(E::InvalidCleanupExit)?;
            if from.ordinal >= to.ordinal
                || !ancestor(&to.scope, &from.scope)
                || !to.visible.is_subset(&from.visible)
            {
                return Err(E::InvalidCleanupExit);
            }
        }
        Ok(())
    }
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

impl Visitor for Lexical<'_> {
    type Error = E;
    fn place(&mut self, place: &CPlace, _access: Access) -> Result<(), E> {
        match place.kind() {
            CPlaceKind::Local(value) => self.local(value)?,
            CPlaceKind::Parameter(value) if value.function() != self.function => {
                return Err(E::WrongLexicalOwner);
            }
            _ => {}
        }
        Ok(())
    }

    fn value(&mut self, value: &CValue) -> Result<(), E> {
        if let CValueKind::Convert {
            conversion: CConversion::AllocationRestore(value),
            ..
        } = value.kind()
        {
            if !ancestor(value.scope(), self.scope()) {
                return Err(E::WrongLexicalOwner);
            }
            if let super::super::CAllocationShape::Elements(count) = value.shape() {
                self.local(count.local())?;
            }
            match value.allocator() {
                CAllocatorSource::Default => {}
                CAllocatorSource::Local(value) => self.local(value)?,
                CAllocatorSource::Parameter(value) => {
                    if value.function() != self.function {
                        return Err(E::WrongLexicalOwner);
                    }
                }
            }
        }
        Ok(())
    }
}
