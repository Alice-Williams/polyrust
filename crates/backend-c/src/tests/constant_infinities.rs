//! Infinity inventory and syntax facts must not masquerade as finite values.
use super::{CNumber, evaluate, scalar_constant_number};
use crate::ast::{
    CExpressions, CKnownConstant, CRegistry, CScalarConstantValue, CScalarType, CUnaryOperator,
};
use crate::ownership::{CSafetyError, layout::Layouts};
use portable_binary64::Binary64Sign;

#[test]
fn infinity_syntax_and_import_facts_keep_exact_sign_and_reject_integer_conversion() {
    let registry = CRegistry::new();
    let ast = CExpressions::new(&registry);
    let positive = ast.known_constant(CKnownConstant::DoubleInfinity);
    let negative = ast.unary(CUnaryOperator::Negate, positive.clone()).unwrap();
    for (sign, expression, expected) in [
        (Binary64Sign::Positive, positive, 0x7ff0000000000000),
        (Binary64Sign::Negative, negative, 0xfff0000000000000),
    ] {
        let syntax = evaluate(&mut Layouts::new(&registry), &expression).unwrap();
        let imported = scalar_constant_number(&CScalarConstantValue::Infinity(sign)).unwrap();
        for number in [syntax, imported] {
            let CNumber::Double(value) = number else {
                panic!("double")
            };
            assert_eq!(value.to_bits(), expected);
            assert!(!value.is_finite());
            assert!(!value.is_nan());
            for ty in [CScalarType::I32, CScalarType::I64, CScalarType::U64] {
                assert!(number.convert(ty).is_err());
            }
        }
    }
    assert_eq!(
        super::known_values::known(CKnownConstant::DoubleInfinity),
        Err(CSafetyError::ExpectedNumericConstant)
    );
}
