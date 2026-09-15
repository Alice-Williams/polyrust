//! Narrow owned-operation admission; not whole-body ownership certification.
#[allow(dead_code)]
pub(crate) mod boxed_record;
mod construction;
#[allow(dead_code)]
pub(crate) mod scalar_record;
mod standard_box;
// Each proof driver exercises its selected source capabilities.
#[allow(dead_code)]
pub(crate) mod record;
mod slots;

pub(crate) use construction::{BoxConstructionInput, ConstructionError, OwnedBoxConstruction};
pub(crate) use slots::Builder;
