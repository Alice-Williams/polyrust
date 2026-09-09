//! Shift obligations use the promoted left width and every possible count.
use super::{CScalarRange as Range, CTransfer, integer::IntegerRange, integer_operations::extrema};
use crate::ast::{CBinaryOperator as B, CScalarRepresentation as R};
use crate::ownership::{CSafetyError as E, constants::limits};

pub(super) fn binary(op: B, left: IntegerRange, right: IntegerRange) -> Result<CTransfer, E> {
    let width = match left.ty().representation() {
        R::Signed(width) | R::Unsigned(width) => width.bits(),
        _ => unreachable!("promoted left shift operand"),
    };
    if right.min() < 0 || right.max() >= i128::from(width) {
        return Err(E::InvalidShift);
    }
    let (a, b, c, d) = (
        left.min(),
        left.max(),
        right.min() as u32,
        right.max() as u32,
    );
    let (min, max) = if op == B::ShiftRight {
        extrema([a >> c, a >> d, b >> c, b >> d])
    } else {
        if a < 0 {
            return Err(E::InvalidShift);
        }
        // The widest admitted left value shifted by at most 63 fits u128.
        let (min, max) = ((a as u128) << c, (b as u128) << d);
        if max > limits(left.ty())?.1 as u128 {
            return match left.ty().representation() {
                R::Unsigned(_) => Ok(CTransfer::MayWrap(Range::full(left.ty())?)),
                R::Signed(_) => Err(E::InvalidShift),
                _ => unreachable!(),
            };
        }
        (min as i128, max as i128)
    };
    Ok(CTransfer::NonWrapping(Range::Integer(
        IntegerRange::checked(left.ty(), min, max)?,
    )))
}
