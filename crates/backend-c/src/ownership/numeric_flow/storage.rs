//! Numeric identities are authenticated storage paths, never source spellings.
use crate::ast::{
    CObjectType, CObjectTypeKind, CPlace, CPlaceKind, CRegistry, CScalarType, CValue, CValueKind,
    contextual::{
        access_statements,
        access_walk::{Access, Visitor},
    },
};
use crate::ownership::{CSafetyError as E, context_facts::ContextFacts};
use std::collections::BTreeSet;

pub(super) use crate::ownership::paths::{Key, Root};

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
