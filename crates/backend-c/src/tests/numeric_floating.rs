//! False ordered predicates preserve NaN unless a current predicate excludes it.
use super::numeric_fixture::Fixture;
use super::*;
use crate::dialect::CKnownCall;

fn double(f: &Fixture, value: CValue) -> CValue {
    f.values()
        .numeric_conversion(CScalarType::F64, value)
        .unwrap()
}

#[test]
fn rederiving_a_materialized_predicate_does_not_reexecute_its_old_expression() {
    let mut f = Fixture::new(&[CScalarType::F64]);
    let flag = f.local(CScalarType::Int, "flag");
    let sum = f.binary(CBinaryOperator::Add, f.input(0), f.input(0));
    let call = f
        .values()
        .call_value(f.values().known(CKnownCall::IsNan), vec![sum])
        .unwrap();
    let branch = f.branch(f.boolean(f.read(&flag)), vec![], vec![]);
    f.check(vec![f.declare(&flag, call), branch]).unwrap();
}

#[test]
fn opposite_zero_signs_do_not_prune_a_numerically_equal_branch() {
    for operator in [CBinaryOperator::Equal, CBinaryOperator::NotEqual] {
        let mut f = Fixture::new(&[CScalarType::Int]);
        let zero = f.local(CScalarType::F64, "negative_zero");
        let negative = f
            .values()
            .unary(CUnaryOperator::Negate, double(&f, f.int(0)))
            .unwrap();
        let comparison = f.compare(operator, f.read(&zero), double(&f, f.int(0)));
        let condition = if operator == CBinaryOperator::Equal {
            comparison
        } else {
            not(&f, comparison)
        };
        let unsafe_division = f.discard(f.binary(CBinaryOperator::Divide, f.int(1), f.input(0)));
        let branch = f.branch(condition, vec![unsafe_division], vec![]);
        assert_eq!(
            f.check(vec![f.declare(&zero, negative), branch]),
            Err(CSafetyError::DivisionByZero)
        );
    }
}
fn not(f: &Fixture, value: CValue) -> CValue {
    f.boolean(f.values().unary(CUnaryOperator::LogicalNot, value).unwrap())
}
fn and(f: &Fixture, left: CValue, right: CValue) -> CValue {
    f.boolean(f.binary(CBinaryOperator::LogicalAnd, left, right))
}

#[test]
fn ordered_finite_guards_prove_exact_exclusive_float_cast_boundaries() {
    for inclusive_upper in [false, true] {
        let mut f = Fixture::new(&[CScalarType::F64]);
        let lower = f.compare(
            CBinaryOperator::GreaterEqual,
            f.input(0),
            double(&f, f.int(i32::MIN)),
        );
        let upper = f.compare(
            if inclusive_upper {
                CBinaryOperator::LessEqual
            } else {
                CBinaryOperator::Less
            },
            f.input(0),
            double(&f, f.size(1 << 31)),
        );
        let cast = f.discard(
            f.values()
                .numeric_conversion(CScalarType::Int, f.input(0))
                .unwrap(),
        );
        let branch = f.branch(and(&f, lower, upper), vec![cast], vec![]);
        let result = f.check(vec![branch]);
        if inclusive_upper {
            assert_eq!(result, Err(CSafetyError::IntegerRange));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn materialized_isnan_relation_is_required_for_false_ordered_guards() {
    for nan_guard in [false, true] {
        let mut f = Fixture::new(&[CScalarType::F64]);
        let result = f.local(CScalarType::Int, "nan_result");
        let call = f
            .values()
            .call_value(f.values().known(CKnownCall::IsNan), vec![f.input(0)])
            .unwrap();
        let lower = not(
            &f,
            f.compare(
                CBinaryOperator::Less,
                f.input(0),
                double(&f, f.int(i32::MIN)),
            ),
        );
        let upper = not(
            &f,
            f.compare(
                CBinaryOperator::GreaterEqual,
                f.input(0),
                double(&f, f.size(1 << 31)),
            ),
        );
        let mut condition = and(&f, lower, upper);
        if nan_guard {
            condition = and(&f, not(&f, f.boolean(f.read(&result))), condition);
        }
        let cast = f.discard(
            f.values()
                .numeric_conversion(CScalarType::Int, f.input(0))
                .unwrap(),
        );
        let branch = f.branch(condition, vec![cast], vec![]);
        let checked = f.check(vec![f.declare(&result, call), branch]);
        if nan_guard {
            checked.unwrap();
        } else {
            assert_eq!(checked, Err(CSafetyError::IntegerRange));
        }
    }
}

#[test]
fn changing_a_predicate_operand_kills_its_materialized_relation() {
    let mut f = Fixture::new(&[CScalarType::F64]);
    let result = f.local(CScalarType::Int, "nan_result");
    let values = f.values();
    let input = values.parameter(f.parameters[0].clone()).unwrap();
    let initial = f.ast().assign(input.clone(), double(&f, f.int(3))).unwrap();
    let call = values
        .call_value(values.known(CKnownCall::IsNan), vec![f.input(0)])
        .unwrap();
    let nan = f.binary(
        CBinaryOperator::Divide,
        double(&f, f.int(0)),
        double(&f, f.int(0)),
    );
    let overwrite = f.ast().assign(input, nan).unwrap();
    let cast = f.discard(
        values
            .numeric_conversion(CScalarType::Int, f.input(0))
            .unwrap(),
    );
    let condition = not(&f, f.boolean(f.read(&result)));
    let branch = f.branch(condition, vec![cast], vec![]);
    assert_eq!(
        f.check(vec![initial, f.declare(&result, call), overwrite, branch]),
        Err(CSafetyError::IntegerRange)
    );
}

#[test]
fn known_predicate_results_are_not_invented_as_zero_or_one() {
    for known in [CKnownCall::IsNan, CKnownCall::SignBit] {
        let mut f = Fixture::new(&[CScalarType::F64]);
        let result = f.local(CScalarType::Int, "predicate_result");
        let call = f
            .values()
            .call_value(f.values().known(known), vec![f.input(0)])
            .unwrap();
        let bad = f.discard(f.binary(CBinaryOperator::Divide, f.int(1), f.int(0)));
        let branch = f.branch(
            f.compare(CBinaryOperator::Greater, f.read(&result), f.int(1)),
            vec![bad],
            vec![],
        );
        assert_eq!(
            f.check(vec![f.declare(&result, call), branch]),
            Err(CSafetyError::DivisionByZero)
        );
    }
}

#[test]
fn nan_on_the_other_operand_prevents_false_comparison_refinement() {
    let mut f = Fixture::new(&[CScalarType::F64, CScalarType::F64]);
    let cast = f.discard(
        f.values()
            .numeric_conversion(CScalarType::Int, f.input(0))
            .unwrap(),
    );
    let condition = not(&f, f.compare(CBinaryOperator::Less, f.input(0), f.input(1)));
    let branch = f.branch(condition, vec![cast], vec![]);
    assert_eq!(f.check(vec![branch]), Err(CSafetyError::IntegerRange));
}
