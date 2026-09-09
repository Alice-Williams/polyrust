//! Bounds precede the actual index operation, including address-only indexing.
use crate::ast::{CIndexBase, CObjectTypeKind, CPlaceKind};
use crate::ownership::{CSafetyError as E, numeric_flow::NumericFacts};

pub(super) fn check(numeric: &NumericFacts<'_>) -> Result<(), E> {
    for observation in numeric.indices() {
        let CPlaceKind::Index { base, .. } = observation.place().kind() else {
            return Err(E::InvalidNumericSite);
        };
        let CIndexBase::Array(base) = base else {
            return Err(E::UnprovedPointerExtent);
        };
        let ty = base.ty().canonical();
        let CObjectTypeKind::Array { length, .. } = ty.kind() else {
            return Err(E::InvalidNumericSite);
        };
        let (min, max) = observation.nonwrapping_bounds()?;
        if min < 0 || max >= i128::from(length.get()) {
            return Err(E::IndexOutOfBounds);
        }
    }
    Ok(())
}
