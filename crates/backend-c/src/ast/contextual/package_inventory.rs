//! Compare actual defining occurrences with every authoritative registration.

use super::super::{
    CAggregateRef, CBlock, CDeclarationKind, CDefinitionKind, CFileItem, CLinkage, CRegistry,
    CSourceFile, CStatement, CStatementKind, registry::CRegistered,
};
use super::CContextError as E;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn check(registry: &CRegistry, files: &[CSourceFile]) -> Result<(), E> {
    let mut seen = Occurrences::default();
    let mut actual_files = BTreeSet::new();
    for file in files {
        if !actual_files.insert(file.identity()) {
            return Err(E::DuplicateOccurrence);
        }
        for item in file.items() {
            seen.item(item)?;
        }
    }
    if actual_files != registry.registered_files().collect() {
        return Err(E::MissingRegistrationOccurrence);
    }
    for node in registry.contextual_inventory() {
        let requires_definition = match node {
            CRegistered::Struct(value) => registry
                .members(&CAggregateRef::Struct(value.clone()))?
                .is_some(),
            CRegistered::Union(value) => registry
                .members(&CAggregateRef::Union(value.clone()))?
                .is_some(),
            // These registrations carry additional ownership/callable proof
            // obligations for 02D. This diagnostic pass cannot discharge them.
            CRegistered::Allocation(_)
            | CRegistered::Witness(_)
            | CRegistered::Table(_)
            | CRegistered::Adapter(_) => continue,
            _ => true,
        };
        if !seen.declared.contains(&node) || (requires_definition && !seen.defined.contains(&node))
        {
            return Err(E::MissingRegistrationOccurrence);
        }
    }
    Ok(())
}

#[derive(Default)]
struct Occurrences<'a> {
    declared: BTreeSet<CRegistered<'a>>,
    defined: BTreeSet<CRegistered<'a>>,
    linkage: BTreeMap<CRegistered<'a>, CLinkage>,
}

impl<'a> Occurrences<'a> {
    fn define(&mut self, node: CRegistered<'a>) -> Result<(), E> {
        self.declared.insert(node);
        if !self.defined.insert(node) {
            return Err(E::DuplicateOccurrence);
        }
        Ok(())
    }

    fn linkage(&mut self, node: CRegistered<'a>, linkage: CLinkage) -> Result<(), E> {
        if self
            .linkage
            .insert(node, linkage)
            .is_some_and(|old| old != linkage)
        {
            return Err(E::LinkageMismatch);
        }
        Ok(())
    }

    fn aggregate(owner: &'a CAggregateRef) -> CRegistered<'a> {
        match owner {
            CAggregateRef::Struct(value) => CRegistered::Struct(value),
            CAggregateRef::Union(value) => CRegistered::Union(value),
        }
    }

    fn item(&mut self, item: &'a CFileItem) -> Result<(), E> {
        match item {
            CFileItem::Declaration(value) => match value.kind() {
                CDeclarationKind::ForwardTag(owner) => {
                    self.declared.insert(Self::aggregate(owner));
                }
                CDeclarationKind::Typedef(value) => self.define(CRegistered::Typedef(value))?,
                CDeclarationKind::Aggregate { owner, members } => {
                    self.define(Self::aggregate(owner))?;
                    for member in members {
                        self.define(CRegistered::Member(member))?;
                    }
                }
                CDeclarationKind::Enum { owner, values } => {
                    self.define(CRegistered::Enum(owner))?;
                    for value in values {
                        self.define(CRegistered::Enumerator(value))?;
                    }
                }
                CDeclarationKind::FunctionPrototype { function, linkage } => {
                    self.declared.insert(CRegistered::Function(function));
                    self.linkage(CRegistered::Function(function), *linkage)?;
                }
                CDeclarationKind::ObjectDeclaration(value) => {
                    self.declared.insert(CRegistered::Object(value));
                    self.linkage(CRegistered::Object(value), CLinkage::External)?;
                }
            },
            CFileItem::Definition(value) => match value.kind() {
                CDefinitionKind::Function {
                    function,
                    linkage,
                    parameters,
                    body,
                } => {
                    self.define(CRegistered::Function(function))?;
                    self.linkage(CRegistered::Function(function), *linkage)?;
                    for parameter in parameters {
                        self.define(CRegistered::Parameter(parameter))?;
                    }
                    self.block(body)?;
                }
                CDefinitionKind::Object {
                    object, linkage, ..
                } => {
                    self.define(CRegistered::Object(object))?;
                    self.linkage(CRegistered::Object(object), *linkage)?;
                }
            },
            CFileItem::Comment(_) | CFileItem::StaticAssert(_) => {}
        }
        Ok(())
    }

    fn block(&mut self, block: &'a CBlock) -> Result<(), E> {
        self.define(CRegistered::Scope(block.scope()))?;
        for statement in block.statements() {
            self.statement(statement)?;
        }
        Ok(())
    }

    fn statement(&mut self, statement: &'a CStatement) -> Result<(), E> {
        match statement.kind() {
            CStatementKind::Block(block) => self.block(block)?,
            CStatementKind::Declare(value) => self.define(CRegistered::Local(value.local()))?,
            CStatementKind::If {
                then_block,
                else_block,
                ..
            } => {
                self.block(then_block)?;
                self.block(else_block)?;
            }
            CStatementKind::BoundedLoop { identity, body, .. } => {
                self.define(CRegistered::Loop(identity))?;
                self.block(body)?;
            }
            CStatementKind::Switch {
                identity,
                arms,
                default,
                ..
            } => {
                self.define(CRegistered::Switch(identity))?;
                for arm in arms {
                    self.block(arm.body())?;
                }
                self.block(default)?;
            }
            CStatementKind::Label {
                identity,
                statement,
            } => {
                self.define(CRegistered::CleanupExit(identity))?;
                self.statement(statement)?;
            }
            CStatementKind::Empty
            | CStatementKind::Assign { .. }
            | CStatementKind::Evaluate(_)
            | CStatementKind::Discard(_)
            | CStatementKind::Break(_)
            | CStatementKind::Continue(_)
            | CStatementKind::Return(_)
            | CStatementKind::CleanupJump(_) => {}
        }
        Ok(())
    }
}
