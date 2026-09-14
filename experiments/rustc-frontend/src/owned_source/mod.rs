//! Narrow owned-operation admission; not whole-body ownership certification.
mod construction;
mod slots;

pub(crate) use construction::{BoxConstructionInput, ConstructionError, OwnedBoxConstruction};
pub(crate) use slots::Builder;
