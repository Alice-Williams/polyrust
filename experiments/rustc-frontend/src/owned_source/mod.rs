//! Narrow owned-operation admission; not whole-body ownership certification.
mod construction;
// Each proof driver exercises its selected source capabilities.
#[allow(dead_code)]
pub(crate) mod record;
mod slots;

pub(crate) use construction::{BoxConstructionInput, ConstructionError, OwnedBoxConstruction};
pub(crate) use slots::Builder;
