//! Case/default numeric restrictions use C promotion, not literal spelling.
use super::{contextual_reconstruction::key, numeric_fixture::Fixture, *};

fn selection(
    f: &mut Fixture,
    cases: Vec<CCaseConstant>,
    yes: Vec<CStatement>,
    no: Vec<CStatement>,
) -> CStatement {
    let identity = f.registry.register_switch(&f.scope, key("choice")).unwrap();
    let selected = f
        .registry
        .register_scope(&f.function, Some(&f.scope), key("selected"))
        .unwrap();
    let fallback = f
        .registry
        .register_scope(&f.function, Some(&f.scope), key("fallback"))
        .unwrap();
    let exit = f
        .ast()
        .break_statement(CBreakTarget::Switch(identity.clone()))
        .unwrap();
    let mut yes = yes;
    let mut no = no;
    yes.push(exit.clone());
    no.push(exit);
    f.ast()
        .switch_statement(
            identity,
            f.input(0),
            vec![
                f.ast()
                    .switch_arm(cases, f.ast().block(selected, yes).unwrap())
                    .unwrap(),
            ],
            f.ast().block(fallback, no).unwrap(),
        )
        .unwrap()
}

#[test]
fn default_excludes_every_selected_case_without_filling_the_zero_hole() {
    for guarded in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Int]);
        let divide = f.discard(f.binary(CBinaryOperator::Divide, f.int(1), f.input(0)));
        let cases = vec![CCaseConstant::Signed(CSignedLiteral::Int(if guarded {
            0
        } else {
            1
        }))];
        let switch = selection(&mut f, cases, vec![], vec![divide]);
        let result = f.check(vec![switch]);
        if guarded {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CSafetyError::DivisionByZero));
        }
    }
}

#[test]
fn signed_high_bit_case_conversion_matches_the_promoted_discriminant() {
    let mut f = Fixture::new(&[CScalarType::Int]);
    let negation = f.discard(
        f.values()
            .unary(CUnaryOperator::Negate, f.input(0))
            .unwrap(),
    );
    let cases = vec![CCaseConstant::Unsigned(CUnsignedLiteral::U32(1_u32 << 31))];
    let switch = selection(&mut f, cases, vec![negation], vec![]);
    assert_eq!(f.check(vec![switch]), Err(CSafetyError::SignedOverflow));
}
