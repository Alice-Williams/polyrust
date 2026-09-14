//! Authenticate storage origins and derive member/index qualification anew.

use super::super::{CIndexBase, CPlace, CPlaceKind};
use super::rebuild_expression_work::{self as work, Built, Children, Node};
use super::{CContextError as E, Recheck};

impl Recheck<'_> {
    pub(super) fn place(&self, place: &CPlace) -> Result<CPlace, E> {
        match work::rebuild(self, Node::Place(place))? {
            Built::Place(place) => Ok(*place),
            Built::Value(_) => Err(E::StoredStructureMismatch),
        }
    }

    pub(super) fn place_node(&self, place: &CPlace, children: &mut Children) -> Result<CPlace, E> {
        let ast = &self.expressions;
        let rebuilt = match place.kind() {
            CPlaceKind::Local(value) => ast.local(value.clone())?,
            CPlaceKind::Parameter(value) => ast.parameter(value.clone())?,
            CPlaceKind::Global(value) => ast.global(value.clone())?,
            CPlaceKind::Member { member, .. } => {
                ast.member(work::place(children)?, member.clone())?
            }
            CPlaceKind::Dereference(_) => ast.dereference(work::value(children)?)?,
            CPlaceKind::Index { base, .. } => {
                let base = match base {
                    CIndexBase::Array(_) => CIndexBase::Array(Box::new(work::place(children)?)),
                    CIndexBase::Pointer(_) => CIndexBase::Pointer(Box::new(work::value(children)?)),
                };
                ast.index(base, work::value(children)?)?
            }
        };
        if place.ty() != rebuilt.ty() || place.brand != rebuilt.brand {
            return Err(E::StoredStructureMismatch);
        }
        Ok(rebuilt)
    }
}
