//! Allocation storage is either one C object or a sequence, never a fake array.
use crate::ast::{CBufferCountRef, CObjectType};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ownership) enum Shape {
    Object(CObjectType),
    Elements {
        element: CObjectType,
        count: CBufferCountRef,
    },
}
impl Shape {
    pub(in crate::ownership) fn object_type(&self) -> Option<&CObjectType> {
        match self {
            Self::Object(ty) => Some(ty),
            Self::Elements { .. } => None,
        }
    }
}
