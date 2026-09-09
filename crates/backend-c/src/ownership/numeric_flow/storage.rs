//! Numeric identities are authenticated storage paths, never source spellings.
use crate::ast::{
    CIndexBase, CLocalRef, CMemberRef, CObjectRef, CObjectType, CObjectTypeKind, CParameterRef,
    CPlace, CPlaceKind, CRegistry, CScalarType, CScopeRef, CValue, CValueKind,
    contextual::{
        access_statements,
        access_walk::{Access, Visitor},
    },
};
use crate::ownership::{
    CSafetyError as E, constants, context_facts::ContextFacts, layout::Layouts,
};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Root {
    Local(CLocalRef),
    Parameter(CParameterRef),
    Global(CObjectRef),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Selector {
    Member(Box<CMemberRef>),
    Index(u64),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Key {
    root: Root,
    selectors: Vec<Selector>,
}

impl Root {
    pub(super) fn place(place: &CPlace) -> Option<Self> {
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
    pub(super) fn leaves(&self, scope: &CScopeRef) -> bool {
        matches!(self, Self::Local(value) if value.scope() == scope)
    }
    pub(super) fn exposed(&self, addresses: &BTreeSet<Self>) -> bool {
        matches!(self, Self::Global(_)) || addresses.contains(self)
    }
}
impl Key {
    pub(super) fn local(local: &CLocalRef) -> Self {
        Self {
            root: Root::Local(local.clone()),
            selectors: vec![],
        }
    }
    pub(super) fn root(&self) -> &Root {
        &self.root
    }
    pub(super) fn rebase(&self, from: &Self, to: &Self) -> Option<Self> {
        if self.root != from.root || !self.selectors.starts_with(&from.selectors) {
            return None;
        }
        let mut key = to.clone();
        key.selectors
            .extend_from_slice(&self.selectors[from.selectors.len()..]);
        Some(key)
    }
    pub(super) fn whole_root(&self) -> bool {
        self.selectors.is_empty()
    }
    pub(super) fn place(place: &CPlace, layouts: &mut Layouts<'_>) -> Option<Self> {
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
                key.selectors
                    .push(Selector::Index(u64::try_from(index).ok()?));
                Some(key)
            }
            CPlaceKind::Dereference(_)
            | CPlaceKind::Index {
                base: CIndexBase::Pointer(_),
                ..
            } => None,
        }
    }
    pub(super) fn member(&self, member: &CMemberRef) -> Self {
        let mut next = self.clone();
        next.selectors
            .push(Selector::Member(Box::new(member.clone())));
        next
    }
    pub(super) fn index(&self, index: u64) -> Self {
        let mut next = self.clone();
        next.selectors.push(Selector::Index(index));
        next
    }
}

pub(super) fn scalar(registry: &CRegistry, ty: &CObjectType) -> Result<Option<CScalarType>, E> {
    Ok(match ty.canonical().kind() {
        CObjectTypeKind::Scalar(ty) => Some(*ty),
        CObjectTypeKind::Enum(value) => {
            let values = registry.enumerators(value)?.ok_or(E::IncompleteLayout)?;
            Some(if values.iter().any(|value| value.value() < 0) {
                CScalarType::Int
            } else {
                CScalarType::U32
            })
        }
        _ => None,
    })
}

struct Collect {
    roots: BTreeSet<Root>,
    addresses_only: bool,
}
impl Visitor for Collect {
    type Error = E;
    fn place(&mut self, place: &CPlace, _access: Access) -> Result<(), E> {
        if (!self.addresses_only || matches!(place.kind(), CPlaceKind::Global(_)))
            && let Some(root) = Root::place(place)
        {
            self.roots.insert(root);
        }
        Ok(())
    }
    fn value(&mut self, value: &CValue) -> Result<(), E> {
        // The walker's Address access also visits non-reading aggregate bases.
        // Only actual address-taking syntax exposes their storage to a call.
        if self.addresses_only
            && let CValueKind::AddressOf(place) = value.kind()
            && let Some(root) = Root::place(place)
        {
            self.roots.insert(root);
        }
        Ok(())
    }
}
pub(super) fn addresses(context: &ContextFacts<'_>) -> Result<BTreeSet<Root>, E> {
    let mut collect = Collect {
        roots: BTreeSet::new(),
        addresses_only: true,
    };
    for file in context.files() {
        access_statements::file(&mut collect, file)?;
    }
    Ok(collect.roots)
}
pub(super) fn dependencies(value: &CValue) -> Result<BTreeSet<Root>, E> {
    let mut collect = Collect {
        roots: BTreeSet::new(),
        addresses_only: false,
    };
    crate::ast::contextual::access_walk::expression(&mut collect, value)?;
    Ok(collect.roots)
}
