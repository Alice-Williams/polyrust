//! Typed guarded normalization; mutants distinguish safety from semantic shape.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(in crate::dialect::shared) enum Operation {
    Add,
    Subtract,
}
impl Operation {
    fn operator(self) -> CBinaryOperator {
        match self {
            Self::Add => CBinaryOperator::Add,
            Self::Subtract => CBinaryOperator::Subtract,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::dialect::shared) enum Variant {
    Valid,
    MissingGuard,
    ReversedGuard,
    HighLimit,
    LowLimit,
    WrongGuardValue,
    SignedArithmetic,
    UnguardedCast,
    MissingNormalization,
    WrongOperand,
    WrongResult,
    NestedComplement(usize),
    NestedMultiply,
    BoundaryLiterals,
    ReversedOperands,
    WrongOperation,
}

pub(in crate::dialect::shared) fn build(
    e: &CExpressions<'_>,
    s: &CStatements<'_>,
    locals: &[CLocalRef; 3],
    inputs: Vec<CValue>,
    operation: Operation,
    variant: Variant,
) -> (Vec<CStatement>, CValue) {
    let CObjectTypeKind::Scalar(signed) = inputs[0].ty().kind() else {
        panic!("signed")
    };
    let signed = *signed;
    let unsigned = if signed == CScalarType::I32 {
        CScalarType::U32
    } else {
        CScalarType::U64
    };
    let max = if signed == CScalarType::I32 {
        i32::MAX as u64
    } else {
        i64::MAX as u64
    };
    let signed_literal = |n| {
        e.literal(CLiteral::Signed(match signed {
            CScalarType::I32 => CSignedLiteral::I32(n as i32),
            CScalarType::I64 => CSignedLiteral::I64(n),
            _ => panic!("width"),
        }))
        .unwrap()
    };
    let unsigned_literal = |n| {
        e.literal(CLiteral::Unsigned(match unsigned {
            CScalarType::U32 => CUnsignedLiteral::U32(n as u32),
            CScalarType::U64 => CUnsignedLiteral::U64(n),
            _ => panic!("width"),
        }))
        .unwrap()
    };
    let mut statements: Vec<_> = inputs
        .into_iter()
        .zip(locals)
        .map(|(input, local)| {
            s.declare(
                local.clone(),
                Some(e.expression_initializer(input).unwrap()),
            )
            .unwrap()
        })
        .collect();
    let read = |index: usize| e.read(e.local(locals[index].clone()).unwrap()).unwrap();
    let mut left = read(0);
    let mut right = if matches!(variant, Variant::WrongOperand) {
        read(0)
    } else {
        read(1)
    };
    if matches!(variant, Variant::ReversedOperands) {
        std::mem::swap(&mut left, &mut right);
    }
    let operator = if matches!(variant, Variant::WrongOperation) {
        match operation {
            Operation::Add => CBinaryOperator::Subtract,
            Operation::Subtract => CBinaryOperator::Add,
        }
    } else {
        operation.operator()
    };
    let sum = if matches!(variant, Variant::SignedArithmetic) {
        let sum = e.binary(operator, left, right).unwrap();
        // I32 promotion produces Int. Normalize before the matching unsigned cast.
        let sum = if signed == CScalarType::I32 {
            e.numeric_conversion(signed, sum).unwrap()
        } else {
            sum
        };
        e.numeric_conversion(unsigned, sum).unwrap()
    } else {
        let mut right = e.numeric_conversion(unsigned, right).unwrap();
        if matches!(variant, Variant::BoundaryLiterals) {
            // Adding UMAX then one is an unsigned identity. Exercise full-range
            // literal spelling without changing the independent expected values.
            let ceiling = if unsigned == CScalarType::U32 {
                u32::MAX as u64
            } else {
                u64::MAX
            };
            right = e
                .binary(CBinaryOperator::Add, right, unsigned_literal(ceiling))
                .unwrap();
            right = e
                .binary(CBinaryOperator::Add, right, unsigned_literal(1))
                .unwrap();
        }
        if let Variant::NestedComplement(count) = variant {
            for _ in 0..count {
                right = e.unary(CUnaryOperator::BitNot, right).unwrap();
            }
        }
        if matches!(variant, Variant::NestedMultiply) {
            right = e
                .binary(CBinaryOperator::Multiply, right, unsigned_literal(1))
                .unwrap();
        }
        e.binary(
            operator,
            e.numeric_conversion(unsigned, left).unwrap(),
            right,
        )
        .unwrap()
    };
    if matches!(variant, Variant::WrongOperand) {
        statements.push(s.discard(read(1)).unwrap());
    }
    statements.push(
        s.declare(
            locals[2].clone(),
            Some(e.expression_initializer(sum).unwrap()),
        )
        .unwrap(),
    );
    let sum = read(2);
    let positive = e.numeric_conversion(signed, sum.clone()).unwrap();
    if matches!(variant, Variant::UnguardedCast) {
        return (statements, positive);
    }
    let limit = match variant {
        Variant::HighLimit => max + 1,
        Variant::LowLimit => max - 1,
        _ => max,
    };
    let guarded = if matches!(variant, Variant::WrongGuardValue) {
        unsigned_literal(0)
    } else {
        sum.clone()
    };
    let comparison = e
        .binary(
            if matches!(variant, Variant::ReversedGuard) {
                CBinaryOperator::Greater
            } else {
                CBinaryOperator::LessEqual
            },
            guarded,
            unsigned_literal(limit),
        )
        .unwrap();
    let condition = if matches!(variant, Variant::MissingGuard) {
        e.literal(CLiteral::Bool(true)).unwrap()
    } else {
        e.numeric_conversion(CScalarType::Bool, comparison).unwrap()
    };
    let complemented = e.unary(CUnaryOperator::BitNot, sum).unwrap();
    let negative = e
        .binary(
            CBinaryOperator::Subtract,
            signed_literal(if matches!(variant, Variant::WrongResult) {
                0
            } else {
                -1
            }),
            e.numeric_conversion(signed, complemented).unwrap(),
        )
        .unwrap();
    let value = e.conditional(condition, positive, negative).unwrap();
    let value = if signed == CScalarType::I32 && !matches!(variant, Variant::MissingNormalization) {
        e.numeric_conversion(CScalarType::I32, value).unwrap()
    } else {
        value
    };
    (statements, value)
}
