//! Interval operators distinguish ordinary unsigned modulo from size evidence.
use super::{CScalarRange as Range, CTransfer, integer::IntegerRange, operators::boolean};
use crate::ast::{CBinaryOperator as B, CScalarRepresentation as R};
use crate::ownership::{CSafetyError as E, constants::limits};

pub(super) fn binary(op: B, left: IntegerRange, right: IntegerRange) -> Result<CTransfer, E> {
    let (a, b, c, d) = (left.min(), left.max(), right.min(), right.max());
    if matches!(
        op,
        B::Equal | B::NotEqual | B::Less | B::LessEqual | B::Greater | B::GreaterEqual
    ) {
        return boolean(compare(op, a, b, c, d));
    }
    if matches!(op, B::Divide | B::Remainder) {
        if right.contains(0) {
            return Err(E::DivisionByZero);
        }
        if matches!(left.ty().representation(), R::Signed(_))
            && left.contains(limits(left.ty())?.0)
            && right.contains(-1)
        {
            return Err(E::SignedOverflow);
        }
    }
    let unsigned = matches!(left.ty().representation(), R::Unsigned(_));
    if unsigned && op == B::Multiply {
        let min = (a as u128) * (c as u128);
        let max = (b as u128) * (d as u128);
        if max > limits(left.ty())?.1 as u128 {
            return Ok(CTransfer::MayWrap(Range::full(left.ty())?));
        }
        return result(left, min as i128, max as i128);
    }
    let (min, max) = match op {
        B::Add => (a + c, b + d),
        B::Subtract => (a - d, b - c),
        B::Multiply => extrema([a * c, a * d, b * c, b * d]),
        B::Divide => extrema([a / c, a / d, b / c, b / d]),
        B::Remainder => {
            let bound = c.abs().max(d.abs()) - 1;
            (
                if a < 0 { a.max(-bound) } else { 0 },
                if b > 0 { b.min(bound) } else { 0 },
            )
        }
        B::BitAnd if a >= 0 && c >= 0 => (0, b.min(d)),
        B::BitOr | B::BitXor if a >= 0 && c >= 0 => {
            let bits = 128 - ((b | d) as u128).leading_zeros();
            (0, (1_i128 << bits) - 1)
        }
        B::BitAnd | B::BitOr | B::BitXor => {
            return Ok(CTransfer::NonWrapping(Range::full(left.ty())?));
        }
        _ => unreachable!("logical, comparison and shift operators handled separately"),
    };
    result(left, min, max)
}

pub(super) fn result(left: IntegerRange, min: i128, max: i128) -> Result<CTransfer, E> {
    let (floor, ceiling) = limits(left.ty())?;
    if min < floor || max > ceiling {
        if matches!(left.ty().representation(), R::Unsigned(_)) {
            return Ok(CTransfer::MayWrap(Range::full(left.ty())?));
        }
        return Err(E::SignedOverflow);
    }
    let lower = IntegerRange::checked(left.ty(), min, min)?;
    let upper = IntegerRange::checked(left.ty(), max, max)?;
    Ok(CTransfer::NonWrapping(Range::Integer(lower.join(upper)?)))
}

pub(super) fn extrema(values: [i128; 4]) -> (i128, i128) {
    (
        *values.iter().min().expect("four endpoints"),
        *values.iter().max().expect("four endpoints"),
    )
}

fn compare(op: B, a: i128, b: i128, c: i128, d: i128) -> Option<bool> {
    match op {
        B::Equal => {
            if b < c || d < a {
                Some(false)
            } else if a == b && c == d {
                Some(a == c)
            } else {
                None
            }
        }
        B::NotEqual => compare(B::Equal, a, b, c, d).map(|value| !value),
        B::Less => {
            if b < c {
                Some(true)
            } else if a >= d {
                Some(false)
            } else {
                None
            }
        }
        B::LessEqual => {
            if b <= c {
                Some(true)
            } else if a > d {
                Some(false)
            } else {
                None
            }
        }
        B::Greater => compare(B::Less, c, d, a, b),
        B::GreaterEqual => compare(B::LessEqual, c, d, a, b),
        _ => unreachable!("arithmetic comparison"),
    }
}
