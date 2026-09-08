//! Case equality uses the switch's actual promoted integer representation.

use super::super::{
    CCaseConstant, CExpressions, CRegistry, CScalarType, CSignedLiteral, CStatementError,
    CSwitchArm, CUnsignedLiteral, CValue,
};
use super::CContextError as E;
use std::collections::BTreeSet;

pub(super) fn check(registry: &CRegistry, value: &CValue, arms: &[CSwitchArm]) -> Result<(), E> {
    let promoted = CExpressions::new(registry)
        .arithmetic_type(value)?
        .integer_promotion()
        .ok_or(CStatementError::ExpectedIntegerSwitch)?;
    let modulus = match promoted {
        CScalarType::Int | CScalarType::U32 => 1_i128 << 32,
        CScalarType::I64 | CScalarType::U64 => 1_i128 << 64,
        _ => return Err(CStatementError::ExpectedIntegerSwitch.into()),
    };
    let mut values = BTreeSet::new();
    for arm in arms {
        for case in arm.cases() {
            let value = match case {
                CCaseConstant::Signed(value) => signed(*value),
                CCaseConstant::Unsigned(value) => unsigned(*value),
                CCaseConstant::Enumerator(value) => {
                    registry.check_enumerator(value)?;
                    i128::from(value.value())
                }
            };
            // Compare exact residues, not spelling or the literal's source
            // width. Original case identities remain in the immutable AST.
            if !values.insert(value.rem_euclid(modulus)) {
                return Err(E::DuplicateCase);
            }
        }
    }
    Ok(())
}

fn signed(value: CSignedLiteral) -> i128 {
    match value {
        CSignedLiteral::PlainChar(value) | CSignedLiteral::I8(value) => i128::from(value),
        CSignedLiteral::I16(value) => i128::from(value),
        CSignedLiteral::Int(value) | CSignedLiteral::I32(value) => i128::from(value),
        CSignedLiteral::I64(value) => i128::from(value),
    }
}
fn unsigned(value: CUnsignedLiteral) -> i128 {
    match value {
        CUnsignedLiteral::U8(value) => i128::from(value),
        CUnsignedLiteral::U16(value) => i128::from(value),
        CUnsignedLiteral::U32(value) => i128::from(value),
        CUnsignedLiteral::U64(value) | CUnsignedLiteral::Size(value) => i128::from(value),
    }
}
