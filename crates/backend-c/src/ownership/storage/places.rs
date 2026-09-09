//! Pointer extents derive from their original typed subobject, never a cast.
use super::{Engine, state::State, values::Pointer};
use crate::ast::{CIndexBase, CObjectTypeKind, CPlace, CPlaceKind};
use crate::ownership::{
    CSafetyError as E,
    paths::{Key, Root, Selector},
};

impl<'ast> Engine<'_, 'ast> {
    pub(super) fn place(&mut self, place: &'ast CPlace, state: &State) -> Result<Key, E> {
        let path = match place.kind() {
            CPlaceKind::Local(_) | CPlaceKind::Parameter(_) | CPlaceKind::Global(_) => {
                Key::from_root(Root::place(place).ok_or(E::UnprovedStorage)?)
            }
            CPlaceKind::Member { base, member } => self.place(base, state)?.member(member),
            CPlaceKind::Dereference(value) => {
                let pointer = self.expression(value, state)?.pointer()?;
                self.target(pointer, place)?
            }
            CPlaceKind::Index { base, index } => {
                self.expression(index, state)?;
                let (first, last) = self.number(index, state)?.extent_bounds()?;
                match base {
                    CIndexBase::Array(base) => {
                        let base = self.place(base, state)?;
                        let ty = base.ty();
                        let CObjectTypeKind::Array { length, .. } = ty.kind() else {
                            return Err(E::UnprovedStorage);
                        };
                        if last >= length.get() {
                            return Err(E::IndexOutOfBounds);
                        }
                        base.interval(first, last)
                    }
                    CIndexBase::Pointer(value) => {
                        let pointer = self.expression(value, state)?.pointer()?;
                        let base = self.target(pointer, place)?;
                        Self::offset(base, first, last)?
                    }
                }
            }
        };
        state.live(&path)?;
        Ok(path)
    }
    fn target(&self, pointer: Pointer, place: &CPlace) -> Result<Key, E> {
        let path = match pointer {
            Pointer::Target(path) => *path,
            Pointer::Null => return Err(E::NullStorage),
            Pointer::Expired => return Err(E::ExpiredStorage),
            Pointer::Unknown | Pointer::Function(_) => return Err(E::UnprovedStorage),
            Pointer::Allocation(_) => return Err(E::UnprovedAllocation),
        };
        if !self
            .registry()
            .pointee_types_match(&path.ty(), place.ty())?
        {
            return Err(E::StorageTypeMismatch);
        }
        Ok(path)
    }
    fn offset(base: Key, first: u64, last: u64) -> Result<Key, E> {
        if let Some((
            parent,
            Selector::Index {
                first: origin_first,
                last: origin_last,
            },
        )) = base.parent()
        {
            let ty = parent.ty();
            let CObjectTypeKind::Array { length, .. } = ty.kind() else {
                return Err(E::UnprovedStorage);
            };
            let first = origin_first.checked_add(first).ok_or(E::IndexOutOfBounds)?;
            let last = origin_last.checked_add(last).ok_or(E::IndexOutOfBounds)?;
            if last >= length.get() {
                return Err(E::IndexOutOfBounds);
            }
            Ok(parent.interval(first, last))
        } else if first == 0 && last == 0 {
            Ok(base)
        } else {
            Err(E::IndexOutOfBounds)
        }
    }
}
