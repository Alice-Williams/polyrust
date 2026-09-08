//! Finite initialized storage paths; no enumeration of huge array bounds.

use super::super::{
    CAggregateRef, CIndexBase, CLocalRef, CMemberRef, CObjectType, CObjectTypeKind, CParameterRef,
    CPlace, CPlaceKind, CRegistry, CScopeRef, CValue,
};
use super::CContextError as E;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Root {
    Local(CLocalRef),
    Parameter(CParameterRef),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Selector {
    Member(Box<CMemberRef>),
    Index(u64),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Path {
    root: Root,
    selectors: Vec<Selector>,
}

impl Path {
    pub(super) fn local(value: &CLocalRef) -> Self {
        Self {
            root: Root::Local(value.clone()),
            selectors: Vec::new(),
        }
    }
    pub(super) fn parameter(value: &CParameterRef) -> Self {
        Self {
            root: Root::Parameter(value.clone()),
            selectors: Vec::new(),
        }
    }
    pub(super) fn place(value: &CPlace) -> Option<Self> {
        match value.kind() {
            CPlaceKind::Local(value) => Some(Self::local(value)),
            CPlaceKind::Parameter(value) => Some(Self::parameter(value)),
            CPlaceKind::Member { base, member } => {
                let mut path = Self::place(base)?;
                path.selectors
                    .push(Selector::Member(Box::new(member.clone())));
                Some(path)
            }
            CPlaceKind::Index {
                base: CIndexBase::Array(base),
                index,
            } => {
                let mut path = Self::place(base)?;
                path.selectors.push(Selector::Index(index_literal(index)?));
                Some(path)
            }
            // Heap and unproved variable-index initialization are 02D facts.
            _ => None,
        }
    }
    fn prefix_of(&self, other: &Self) -> bool {
        self.root == other.root && other.selectors.starts_with(&self.selectors)
    }
    fn ty(&self) -> CObjectType {
        let mut ty = match &self.root {
            Root::Local(value) => value.ty().canonical(),
            Root::Parameter(value) => value.ty().canonical(),
        };
        for selector in &self.selectors {
            ty = match selector {
                Selector::Member(value) => value.ty().canonical(),
                Selector::Index(_) => match ty.kind() {
                    CObjectTypeKind::Array { element, .. } => (**element).clone(),
                    _ => unreachable!("paths derive only from checked array places"),
                },
            };
        }
        ty
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub(super) struct State(BTreeSet<Path>);

impl State {
    pub(super) fn mark(&mut self, path: Path) {
        self.0.insert(path);
    }
    pub(super) fn kill_local(&mut self, local: &CLocalRef) {
        self.0
            .retain(|path| path.root != Root::Local(local.clone()));
    }
    pub(super) fn leave_scope(&mut self, scope: &CScopeRef) {
        self.0
            .retain(|path| !matches!(&path.root, Root::Local(value) if value.scope() == scope));
    }
    pub(super) fn intersect(&self, other: &Self, registry: &CRegistry) -> Result<Self, E> {
        let mut result = Self::default();
        for path in self.0.union(&other.0) {
            if self.covers(path, registry)? && other.covers(path, registry)? {
                result.mark(path.clone());
            }
        }
        Ok(result)
    }
    pub(super) fn covers(&self, path: &Path, registry: &CRegistry) -> Result<bool, E> {
        if self.0.iter().any(|initialized| initialized.prefix_of(path)) {
            return Ok(true);
        }
        match path.ty().kind() {
            CObjectTypeKind::Struct(owner) => {
                self.aggregate(path, &CAggregateRef::Struct(owner.clone()), false, registry)
            }
            CObjectTypeKind::Union(owner) => {
                self.aggregate(path, &CAggregateRef::Union(owner.clone()), true, registry)
            }
            CObjectTypeKind::Array { element: _, length } => {
                let indices: BTreeSet<_> = self
                    .0
                    .iter()
                    .filter(|value| path.prefix_of(value))
                    .filter_map(|value| match value.selectors.get(path.selectors.len()) {
                        Some(Selector::Index(value)) => Some(*value),
                        _ => None,
                    })
                    .collect();
                if u64::try_from(indices.len()) != Ok(length.get()) {
                    return Ok(false);
                }
                for index in indices {
                    if index >= length.get() {
                        return Ok(false);
                    }
                    let mut child = path.clone();
                    child.selectors.push(Selector::Index(index));
                    if !self.covers(&child, registry)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    fn aggregate(
        &self,
        path: &Path,
        owner: &CAggregateRef,
        union: bool,
        registry: &CRegistry,
    ) -> Result<bool, E> {
        let members = registry.members(owner)?.ok_or(E::IncompleteObject)?;
        for member in members {
            let mut child = path.clone();
            child
                .selectors
                .push(Selector::Member(Box::new(member.clone())));
            let covered = self.covers(&child, registry)?;
            if union && covered {
                return Ok(true);
            }
            if !union && !covered {
                return Ok(false);
            }
        }
        Ok(!union)
    }
}

fn index_literal(value: &CValue) -> Option<u64> {
    u64::try_from(super::constant_leaves::integer(value)?).ok()
}
