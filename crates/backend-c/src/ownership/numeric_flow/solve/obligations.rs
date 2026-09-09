//! Numerical checks do not pretend to discharge storage or callable contracts.
use crate::ownership::{
    numeric_flow::{E, Engine, Obligation, storage},
    ranges::NumericLoss,
};
use crate::{
    ast::{CCallableKind, CPlaceKind},
    dialect::CKnownCall,
};

impl Engine<'_> {
    pub(in crate::ownership::numeric_flow) fn check_obligations(&self) -> Result<(), E> {
        for obligation in &self.obligations {
            match &obligation.kind {
                Obligation::Calculation { value, number } => {
                    if storage::scalar(self.registry, value.ty())? != Some(number.domain.ty()) {
                        return Err(E::ExpectedNumericValue);
                    }
                }
                Obligation::Index { place, index } => {
                    let CPlaceKind::Index {
                        index: expression, ..
                    } = place.kind()
                    else {
                        return Err(E::ExpectedNumericValue);
                    };
                    if storage::scalar(self.registry, expression.ty())? != Some(index.domain.ty()) {
                        return Err(E::ExpectedNumericValue);
                    }
                    // Extent/null/provenance obligations remain outstanding.
                }
                Obligation::Call {
                    call,
                    arguments,
                    byte_product,
                } => {
                    let CCallableKind::Known(known) = call.callable().kind() else {
                        continue;
                    };
                    let positions: &[usize] = match known {
                        CKnownCall::Allocate => &[0],
                        CKnownCall::CopyBytes | CKnownCall::CompareBytes => &[2],
                        CKnownCall::WriteBytes => &[1, 2],
                        _ => &[],
                    };
                    for index in positions {
                        if arguments[*index]
                            .as_ref()
                            .is_none_or(|value| !value.losses.is_empty())
                        {
                            return Err(E::UnprovedSizeArithmetic);
                        }
                    }
                    if *known == CKnownCall::WriteBytes
                        && byte_product.as_ref().ok_or(E::ExpectedNumericValue)?.loss
                            != NumericLoss::None
                    {
                        return Err(E::UnprovedSizeArithmetic);
                    }
                }
            }
        }
        Ok(())
    }
}
