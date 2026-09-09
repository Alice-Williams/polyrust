//! Shared authenticated storage identities for numeric and memory analyses.
use crate::ast::{
    CIndexBase, CLocalRef, CMemberRef, CObjectRef, CObjectType, CObjectTypeKind, CParameterRef,
    CPlace, CPlaceKind, CScopeRef,
};
use crate::ownership::{constants, layout::Layouts};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ownership) enum Root {
    Local(CLocalRef),
    Parameter(CParameterRef),
    Global(CObjectRef),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ownership) enum Selector {
    Member(Box<CMemberRef>),
    Index { first: u64, last: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ownership) struct Key {
    root: Root,
    selectors: Vec<Selector>,
}

impl Root {
    pub(in crate::ownership) fn ty(&self) -> CObjectType {
        match self {
            Self::Local(value) => value.ty(),
            Self::Parameter(value) => value.ty(),
            Self::Global(value) => value.ty(),
        }
        .canonical()
    }
    pub(in crate::ownership) fn place(place: &CPlace) -> Option<Self> {
        match place.kind() {
            CPlaceKind::Local(value) => Some(Self::Local(value.clone())),
            CPlaceKind::Parameter(value) => Some(Self::Parameter(value.clone())),
            CPlaceKind::Global(value) => Some(Self::Global(value.clone())),
            CPlaceKind::Member { base, .. }
            | CPlaceKind::Index {
                base: CIndexBase::Array(base),
                ..
            } => Self::place(base),
            CPlaceKind::Dereference(_)
            | CPlaceKind::Index {
                base: CIndexBase::Pointer(_),
                ..
            } => None,
        }
    }
    pub(in crate::ownership) fn leaves(&self, scope: &CScopeRef) -> bool {
        matches!(self, Self::Local(value) if value.scope() == scope)
    }
    pub(in crate::ownership) fn exposed(&self, addresses: &BTreeSet<Self>) -> bool {
        matches!(self, Self::Global(_)) || addresses.contains(self)
    }
}
impl Key {
    pub(in crate::ownership) fn from_root(root: Root) -> Self {
        Self {
            root,
            selectors: vec![],
        }
    }
    pub(in crate::ownership) fn selectors(&self) -> &[Selector] {
        &self.selectors
    }
    pub(in crate::ownership) fn parent(&self) -> Option<(Self, Selector)> {
        let mut parent = self.clone();
        let selector = parent.selectors.pop()?;
        Some((parent, selector))
    }
    pub(in crate::ownership) fn ty(&self) -> CObjectType {
        let mut ty = self.root.ty();
        for selector in &self.selectors {
            ty = match selector {
                Selector::Member(member) => member.ty().canonical(),
                Selector::Index { .. } => match ty.kind() {
                    CObjectTypeKind::Array { element, .. } => element.canonical(),
                    _ => unreachable!("indices derive only from authenticated array paths"),
                },
            };
        }
        ty
    }
    pub(in crate::ownership) fn local(local: &CLocalRef) -> Self {
        Self {
            root: Root::Local(local.clone()),
            selectors: vec![],
        }
    }
    pub(in crate::ownership) fn root(&self) -> &Root {
        &self.root
    }
    pub(in crate::ownership) fn rebase(&self, from: &Self, to: &Self) -> Option<Self> {
        if self.root != from.root || !self.selectors.starts_with(&from.selectors) {
            return None;
        }
        let mut key = to.clone();
        key.selectors
            .extend_from_slice(&self.selectors[from.selectors.len()..]);
        Some(key)
    }
    pub(in crate::ownership) fn whole_root(&self) -> bool {
        self.selectors.is_empty()
    }
    pub(in crate::ownership) fn place(place: &CPlace, layouts: &mut Layouts<'_>) -> Option<Self> {
        match place.kind() {
            CPlaceKind::Local(_) | CPlaceKind::Parameter(_) | CPlaceKind::Global(_) => Some(Self {
                root: Root::place(place)?,
                selectors: vec![],
            }),
            CPlaceKind::Member { base, member } => {
                let mut key = Self::place(base, layouts)?;
                key.selectors
                    .push(Selector::Member(Box::new(member.clone())));
                Some(key)
            }
            CPlaceKind::Index {
                base: CIndexBase::Array(base),
                index,
            } => {
                let index = constants::evaluate(layouts, index)
                    .ok()?
                    .integer()
                    .ok()?
                    .value();
                let mut key = Self::place(base, layouts)?;
                let index = u64::try_from(index).ok()?;
                key.selectors.push(Selector::Index {
                    first: index,
                    last: index,
                });
                Some(key)
            }
            CPlaceKind::Dereference(_)
            | CPlaceKind::Index {
                base: CIndexBase::Pointer(_),
                ..
            } => None,
        }
    }
    pub(in crate::ownership) fn member(&self, member: &CMemberRef) -> Self {
        let mut next = self.clone();
        next.selectors
            .push(Selector::Member(Box::new(member.clone())));
        next
    }
    pub(in crate::ownership) fn index(&self, index: u64) -> Self {
        self.interval(index, index)
    }
    pub(in crate::ownership) fn interval(&self, first: u64, last: u64) -> Self {
        let mut next = self.clone();
        next.selectors.push(Selector::Index { first, last });
        next
    }
}
