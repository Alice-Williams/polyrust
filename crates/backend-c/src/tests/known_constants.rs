//! Library constants retain actual types, not caller-authored names or flags.

use super::registry_nominals::registry;
use crate::ast::{
    CExpressions, CKnownConstant as K, CKnownObject, CObjectType, CPointerTarget, CScalarType as T,
    CValueKind,
};

#[test]
fn every_initial_constant_has_its_pinned_type_and_constant_expression_category() {
    let (registry, _) = registry();
    let ast = CExpressions::new(&registry);
    for (constant, ty) in [
        (K::CharBit, T::Int),
        (K::IntMin, T::Int),
        (K::IntMax, T::Int),
        (K::I32Min, T::Int),
        (K::I32Max, T::Int),
        (K::U32Max, T::U32),
        (K::I64Min, T::I64),
        (K::I64Max, T::I64),
        (K::U64Max, T::U64),
        (K::SizeMax, T::Size),
        (K::FloatRadix, T::Int),
        (K::DoubleMantissaDigits, T::Int),
        (K::DoubleMinExponent, T::Int),
        (K::DoubleMaxExponent, T::Int),
        (K::FloatEvaluationMethod, T::Int),
        (K::EndOfFile, T::Int),
    ] {
        assert!(constant.is_integer_constant_expression());
        let value = ast.known_constant(constant);
        assert_eq!(value.ty(), &CObjectType::scalar(ty));
        assert_eq!(value.kind(), &CValueKind::KnownConstant(constant));
    }
    for constant in [K::StandardInput, K::StandardOutput, K::StandardError] {
        assert!(!constant.is_integer_constant_expression());
        assert_eq!(
            ast.known_constant(constant).ty(),
            &CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::known(
                CKnownObject::File
            ))))
        );
    }
}
