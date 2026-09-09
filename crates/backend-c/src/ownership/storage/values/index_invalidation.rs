//! A saved address cannot silently follow a mutated index binding.
use super::{Cell, Pointer, Root};

impl Cell {
    pub(in crate::ownership::storage) fn invalidate_index(&mut self, root: &Root) {
        match self {
            Self::Pointer(Pointer::Target(path))
                if path.index_touches(|dependency| dependency == root) =>
            {
                *self = Self::Pointer(Pointer::Unknown);
            }
            Self::Record(fields) => fields
                .values_mut()
                .for_each(|cell| cell.invalidate_index(root)),
            Self::Array { default, elements } => {
                default.invalidate_index(root);
                elements
                    .values_mut()
                    .for_each(|cell| cell.invalidate_index(root));
            }
            Self::Union { value, .. } => value.invalidate_index(root),
            _ => {}
        }
    }
}
