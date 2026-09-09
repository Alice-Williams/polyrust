//! Compare actual defining occurrences with every authoritative registration.

use super::super::{
    CAggregateRef, CBlock, CDeclarationKind, CDefinitionKind, CFileItem, CFileRef, CLinkage,
    CRegistry, CSourceFile, CStatement, CStatementKind, registry::CRegistered,
};
use super::CContextError as E;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn check(registry: &CRegistry, files: &[CSourceFile]) -> Result<(), E> {
    super::origin_roles::check(registry, files)?;
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
            CRegistered::OwnerSlot(value) => {
                registry.check_owner_slot(value.local().scope().function(), value)?;
                if !seen.defined.contains(&CRegistered::Local(value.local())) {
                    return Err(E::MissingRegistrationOccurrence);
                }
                // A role refers to the existing local definition, not a second
                // source declaration or proof-authored occurrence.
                continue;
            }
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
        if !seen.observed.contains(&node) || (requires_definition && !seen.defined.contains(&node))
        {
            return Err(E::MissingRegistrationOccurrence);
        }
        if matches!(
            node,
            CRegistered::Struct(_)
                | CRegistered::Union(_)
                | CRegistered::Function(_)
                | CRegistered::Object(_)
        ) && !seen.declared_at_owner.contains(&node)
        {
            return Err(E::MissingRegistrationOccurrence);
        }
    }
    Ok(())
}

#[derive(Default)]
struct Occurrences<'a> {
    observed: BTreeSet<CRegistered<'a>>,
    declared_at_owner: BTreeSet<CRegistered<'a>>,
    defined: BTreeSet<CRegistered<'a>>,
    linkage: BTreeMap<CRegistered<'a>, CLinkage>,
}

impl<'a> Occurrences<'a> {
    fn define(&mut self, node: CRegistered<'a>) -> Result<(), E> {
        self.observed.insert(node);
        if !self.defined.insert(node) {
            return Err(E::DuplicateOccurrence);
        }
        Ok(())
    }

    fn owner_declaration(&mut self, node: CRegistered<'a>, file: &CFileRef) {
        let owner = match node {
            CRegistered::Struct(value) => value.file(),
            CRegistered::Union(value) => value.file(),
            CRegistered::Function(value) => value.file(),
            CRegistered::Object(value) => value.file(),
            _ => return,
        };
        // A moved implementation is not its public/private header declaration.
        // Same-file definitions may declare their own source-local symbol.
        if owner == file {
            self.declared_at_owner.insert(node);
        }
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
                    self.observed.insert(Self::aggregate(owner));
                    self.owner_declaration(Self::aggregate(owner), value.file());
                }
                CDeclarationKind::Typedef(value) => self.define(CRegistered::Typedef(value))?,
                CDeclarationKind::Aggregate { owner, members } => {
                    self.define(Self::aggregate(owner))?;
                    self.owner_declaration(Self::aggregate(owner), value.file());
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
                    self.observed.insert(CRegistered::Function(function));
                    self.owner_declaration(CRegistered::Function(function), value.file());
                    self.linkage(CRegistered::Function(function), *linkage)?;
                }
                CDeclarationKind::ObjectDeclaration(object) => {
                    self.observed.insert(CRegistered::Object(object));
                    self.owner_declaration(CRegistered::Object(object), value.file());
                    self.linkage(CRegistered::Object(object), CLinkage::External)?;
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
                    self.owner_declaration(CRegistered::Function(function), value.file());
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
                    self.owner_declaration(CRegistered::Object(object), value.file());
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
