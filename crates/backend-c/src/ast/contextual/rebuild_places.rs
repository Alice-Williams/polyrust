//! Authenticate storage origins and derive member/index qualification anew.

use super::super::{CIndexBase, CPlace, CPlaceKind};
use super::{CContextError as E, Recheck};

impl Recheck<'_> {
    pub(super) fn place(&self, place: &CPlace) -> Result<CPlace, E> {
        let ast = &self.expressions;
        let rebuilt = match place.kind() {
            CPlaceKind::Local(value) => ast.local(value.clone())?,
            CPlaceKind::Parameter(value) => ast.parameter(value.clone())?,
            CPlaceKind::Global(value) => ast.global(value.clone())?,
            CPlaceKind::Member { base, member } => ast.member(self.place(base)?, member.clone())?,
            CPlaceKind::Dereference(value) => ast.dereference(self.value(value)?)?,
            CPlaceKind::Index { base, index } => {
                let base = match base {
                    CIndexBase::Array(place) => CIndexBase::Array(Box::new(self.place(place)?)),
                    CIndexBase::Pointer(value) => CIndexBase::Pointer(Box::new(self.value(value)?)),
                };
                ast.index(base, self.value(index)?)?
            }
        };
        if place.ty() != rebuilt.ty() || place.brand != rebuilt.brand {
            return Err(E::StoredStructureMismatch);
        }
        Ok(rebuilt)
    }
}
