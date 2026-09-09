//! Shared authenticated storage identities for numeric and memory analyses.
mod element_index;
mod observations;
mod shape;
use crate::ast::{
    CIndexBase, CLocalRef, CMemberRef, CObjectRef, CObjectType, CObjectTypeKind, CParameterRef,
    CPlace, CPlaceKind, CScopeRef,
};
use crate::ownership::{constants, layout::Layouts};
pub(in crate::ownership) use element_index::ElementIndex;
pub(in crate::ownership) use shape::Shape;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ownership) enum Root {
    Local(CLocalRef),
    Parameter(CParameterRef),
    Global(CObjectRef),
    Allocation(Box<super::numeric_flow::AllocationOrigin>, Shape),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ownership) enum Selector {
    Member(Box<CMemberRef>),
    Index { first: u64, last: u64 },
    Element(Box<ElementIndex>),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ownership) struct Key {
    root: Root,
    selectors: Vec<Selector>,
}

impl Root {
    pub(in crate::ownership) fn shape(&self) -> Shape {
        match self {
            Self::Local(value) => Shape::Object(value.ty().canonical()),
            Self::Parameter(value) => Shape::Object(value.ty().canonical()),
            Self::Global(value) => Shape::Object(value.ty().canonical()),
            Self::Allocation(_, shape) => shape.clone(),
        }
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
        let selectors = match root.shape() {
            Shape::Object(_) => vec![],
            Shape::Elements { .. } => vec![Selector::Element(Box::new(ElementIndex::constant(0)))],
        };
        Self { root, selectors }
    }
    pub(in crate::ownership) fn selectors(&self) -> &[Selector] {
        &self.selectors
    }
    pub(in crate::ownership) fn parent(&self) -> Option<(Self, Selector)> {
        let mut parent = self.clone();
        let selector = parent.selectors.pop()?;
        if matches!(selector, Selector::Element(_)) {
            return None;
        }
        Some((parent, selector))
    }
    pub(in crate::ownership) fn ty(&self) -> CObjectType {
        let mut ty = match self.root.shape() {
            Shape::Object(ty) => ty,
            Shape::Elements { element, .. } => element,
        };
        for selector in &self.selectors {
            ty = match selector {
                Selector::Element(_) => ty,
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
        if !self.observes_prefix(from) {
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
    pub(in crate::ownership) fn allocation_base(&self) -> bool {
        matches!(self.root, Root::Allocation(..))
            && (self.whole_root()
                || matches!(
                    self.selectors.as_slice(),
                    [Selector::Element(index)] if index.constant_value() == Some(0)
                ))
    }
    pub(in crate::ownership) fn exact(&self) -> bool {
        self.selectors.iter().all(|selector| match selector {
            Selector::Index { first, last } => first == last,
            Selector::Element(index) => index.exact(),
            Selector::Member(_) => true,
        })
    }
    pub(in crate::ownership) fn index_touches(&self, mut test: impl FnMut(&Root) -> bool) -> bool {
        self.selectors.iter().any(|selector| match selector {
            Selector::Element(index) => index.touches(&mut test),
            _ => false,
        })
    }
    pub(in crate::ownership) fn touches(&self, mut test: impl FnMut(&Root) -> bool) -> bool {
        test(self.root()) || self.index_touches(test)
    }
    pub(in crate::ownership) fn overlaps(&self, other: &Self) -> bool {
        if self.root != other.root {
            return false;
        }
        let mut ty = self.root.shape().object_type().cloned();
        for (left, right) in self.selectors.iter().zip(&other.selectors) {
            if left != right && observations::join_selector(left, right).is_none() {
                return match (left, right) {
                    (
                        Selector::Index { first: a, last: b },
                        Selector::Index { first: c, last: d },
                    ) => a <= d && c <= b,
                    (Selector::Element(left), Selector::Element(right)) => left.overlaps(right),
                    (Selector::Member(_), Selector::Member(_)) => ty
                        .as_ref()
                        .is_some_and(|ty| matches!(ty.kind(), CObjectTypeKind::Union(_))),
                    _ => true,
                };
            }
            ty = match left {
                Selector::Element(_) => match self.root.shape() {
                    Shape::Elements { element, .. } => Some(element),
                    _ => None,
                },
                Selector::Member(member) => Some(member.ty().canonical()),
                Selector::Index { .. } => ty.and_then(|ty| match ty.kind() {
                    CObjectTypeKind::Array { element, .. } => Some(element.canonical()),
                    _ => None,
                }),
            };
        }
        true
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
    pub(in crate::ownership) fn element_selection(&self, index: ElementIndex) -> Option<Self> {
        if !matches!(self.selectors.as_slice(), [Selector::Element(_)]) {
            return None;
        }
        Some(Self {
            root: self.root.clone(),
            selectors: vec![Selector::Element(Box::new(index))],
        })
    }
}
