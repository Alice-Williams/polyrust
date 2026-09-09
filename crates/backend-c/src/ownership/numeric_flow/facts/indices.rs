//! Borrowed views of actual site-validated index observations, not input proofs.
use super::NumericFacts;
use crate::ast::CPlace;
use crate::ownership::{
    CSafetyError as E,
    numeric_flow::{Obligation, state::Number},
};

pub(in crate::ownership) struct IndexObservation<'facts, 'ast> {
    place: &'ast CPlace,
    number: &'facts Number<'ast>,
}

impl<'ast> NumericFacts<'ast> {
    pub(in crate::ownership) fn indices(&self) -> impl Iterator<Item = IndexObservation<'_, 'ast>> {
        let runtime = self.analysis.obligations.iter().filter_map(|obligation| {
            if let Obligation::Index { place, index } = &obligation.kind {
                Some(IndexObservation {
                    place,
                    number: index,
                })
            } else {
                None
            }
        });
        runtime.chain(
            self.static_indices
                .iter()
                .map(|observation| IndexObservation {
                    place: observation.location.place,
                    number: &observation.number,
                }),
        )
    }
}

impl<'ast> IndexObservation<'_, 'ast> {
    pub(in crate::ownership) fn place(&self) -> &'ast CPlace {
        self.place
    }

    pub(in crate::ownership) fn nonwrapping_bounds(&self) -> Result<(i128, i128), E> {
        if !self.number.losses.is_empty() {
            return Err(E::UnprovedSizeArithmetic);
        }
        self.number
            .domain
            .integer_bounds()
            .ok_or(E::ExpectedNumericValue)
    }
}
