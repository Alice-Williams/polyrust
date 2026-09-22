//! Exact width and operand/dataflow shape, independently of C ABI permissiveness.
use super::*;

fn conversion(value: &CValue, ty: CScalarType) -> Option<&CValue> {
    match value.kind() {
        CValueKind::Convert {
            conversion: CConversion::Numeric(target),
            operand,
        } if *target == ty => Some(operand),
        _ => None,
    }
}
fn local_read(value: &CValue, local: &CLocalRef) -> bool {
    matches!(value.kind(), CValueKind::Read(place) if
        matches!(place.kind(), CPlaceKind::Local(found) if found == local))
}
fn correct(source: &f::Fixture) -> bool {
    source.files[1].items().iter().all(|item| {
        let CFileItem::Definition(definition) = item else {
            return false;
        };
        let CDefinitionKind::Function {
            parameters, body, ..
        } = definition.kind()
        else {
            return false;
        };
        let CObjectTypeKind::Scalar(signed) = parameters[0].ty().kind() else {
            return false;
        };
        let (unsigned, limit, minus_one) = match signed {
            CScalarType::I32 => (
                CScalarType::U32,
                CLiteral::Unsigned(CUnsignedLiteral::U32(i32::MAX as u32)),
                CLiteral::Signed(CSignedLiteral::I32(-1)),
            ),
            CScalarType::I64 => (
                CScalarType::U64,
                CLiteral::Unsigned(CUnsignedLiteral::U64(i64::MAX as u64)),
                CLiteral::Signed(CSignedLiteral::I64(-1)),
            ),
            _ => return false,
        };
        let [a, b, sum, result] = body.statements() else {
            return false;
        };
        let (
            CStatementKind::Declare(a),
            CStatementKind::Declare(b),
            CStatementKind::Declare(sum),
            CStatementKind::Return(Some(result)),
        ) = (a.kind(), b.kind(), sum.kind(), result.kind())
        else {
            return false;
        };
        for (local, parameter) in [(a, &parameters[0]), (b, &parameters[1])] {
            let Some(initializer) = local.initializer() else {
                return false;
            };
            let CInitializerKind::Expression(value) = initializer.kind() else {
                return false;
            };
            if !matches!(value.kind(), CValueKind::Read(place) if
                matches!(place.kind(), CPlaceKind::Parameter(found) if found == parameter))
            {
                return false;
            }
        }
        let Some(initializer) = sum.initializer() else {
            return false;
        };
        let CInitializerKind::Expression(value) = initializer.kind() else {
            return false;
        };
        let CValueKind::Binary {
            operator: CBinaryOperator::Add,
            left,
            right,
        } = value.kind()
        else {
            return false;
        };
        if value.ty().kind() != &CObjectTypeKind::Scalar(unsigned)
            || !conversion(left, unsigned).is_some_and(|v| local_read(v, a.local()))
            || !conversion(right, unsigned).is_some_and(|v| local_read(v, b.local()))
        {
            return false;
        }
        if result.ty().kind() != &CObjectTypeKind::Scalar(*signed) {
            return false;
        }
        let result = if *signed == CScalarType::I32 {
            let Some(value) = conversion(result, *signed) else {
                return false;
            };
            if value.ty().kind() != &CObjectTypeKind::Scalar(CScalarType::Int) {
                return false;
            }
            value
        } else {
            result
        };
        let CValueKind::Conditional {
            condition,
            then_value,
            else_value,
        } = result.kind()
        else {
            return false;
        };
        let Some(condition) = conversion(condition, CScalarType::Bool) else {
            return false;
        };
        let CValueKind::Binary {
            operator: CBinaryOperator::LessEqual,
            left,
            right,
        } = condition.kind()
        else {
            return false;
        };
        if !local_read(left, sum.local())
            || !matches!(right.kind(), CValueKind::Literal(value) if *value == limit)
            || !conversion(then_value, *signed).is_some_and(|v| local_read(v, sum.local()))
        {
            return false;
        }
        let CValueKind::Binary {
            operator: CBinaryOperator::Subtract,
            left,
            right,
        } = else_value.kind()
        else {
            return false;
        };
        if !matches!(left.kind(), CValueKind::Literal(value) if *value == minus_one) {
            return false;
        }
        let Some(right) = conversion(right, *signed) else {
            return false;
        };
        matches!(right.kind(), CValueKind::Unary { operator: CUnaryOperator::BitNot, operand }
            if local_read(operand, sum.local()))
    })
}

#[test]
fn wrapping_addition_shape_detects_safe_faults_and_missing_exact_normalization() {
    for width in [CScalarType::I32, CScalarType::I64] {
        let build =
            |variant| fixture::build_widths(701, &[], fixture::Body::Addition(variant), &[width]);
        assert!(correct(&build(Variant::Valid)));
        for variant in [
            Variant::WrongOperand,
            Variant::WrongResult,
            Variant::MissingNormalization,
        ] {
            let source = build(variant);
            api(&source); // Safety can accept semantically wrong but well-defined C.
            assert_eq!(
                correct(&source),
                width == CScalarType::I64 && matches!(variant, Variant::MissingNormalization),
                "{width:?} {variant:?}"
            );
        }
        for variant in [
            Variant::MissingGuard,
            Variant::ReversedGuard,
            Variant::HighLimit,
            Variant::LowLimit,
            Variant::WrongGuardValue,
            Variant::SignedAdd,
            Variant::UnguardedCast,
        ] {
            assert!(!correct(&build(variant)), "{width:?} {variant:?}");
        }
    }
}
