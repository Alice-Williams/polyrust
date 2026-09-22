//! Lossless widening retains range and cannot launder earlier narrowing losses.
use super::*;
use crate::ast::{numeric_fixture::Fixture, *};
use crate::dialect::CKnownCall;
use crate::ownership::numeric_flow::{Obligation, provenance::Origin};

#[test]
fn signed_widening_preserves_the_full_original_signed_range() {
    let f = Fixture::new(&[CScalarType::I32]);
    let widened = f
        .values()
        .numeric_conversion(CScalarType::I64, f.input(0))
        .unwrap();
    let files = [f.source(vec![f.discard(widened)])];
    let facts = NumericFacts::check(&f.registry, &files).unwrap();
    let numbers = facts
        .analysis
        .obligations
        .iter()
        .filter_map(|entry| {
            if let Obligation::Calculation { value, number } = &entry.kind
                && value.ty() == &CObjectType::scalar(CScalarType::I64)
            {
                return Some(number);
            }
            None
        })
        .collect::<Vec<_>>();
    assert_eq!(numbers.len(), 1);
    assert_eq!(
        numbers[0].domain.integer_bounds(),
        Some((i128::from(i32::MIN), i128::from(i32::MAX)))
    );
    assert!(numbers[0].losses.is_empty());
    f.check(vec![
        f.discard(
            f.values()
                .numeric_conversion(CScalarType::I64, f.input(0))
                .unwrap(),
        ),
    ])
    .unwrap();
}

#[test]
fn signed_widening_retains_narrowing_loss_for_indices_and_allocation() {
    for lossy in [false, true] {
        let mut f = Fixture::new(&[CScalarType::U64]);
        let input = if lossy { f.input(0) } else { f.size(7) };
        let narrow = f
            .values()
            .numeric_conversion(CScalarType::U8, input)
            .unwrap();
        let signed = f
            .values()
            .numeric_conversion(CScalarType::I32, narrow)
            .unwrap();
        let widened = f
            .values()
            .numeric_conversion(CScalarType::I64, signed)
            .unwrap();
        let array = f
            .registry
            .register_local(
                &f.scope,
                CDeclarationKey {
                    name: CIdentifier::new("array").unwrap(),
                    origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
                },
                CObjectType::array(
                    CObjectType::scalar(CScalarType::Int),
                    CArrayLength::new(256).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        let place = f
            .values()
            .index(
                CIndexBase::Array(Box::new(f.values().local(array.clone()).unwrap())),
                widened.clone(),
            )
            .unwrap();
        let files = [f.source(vec![
            f.ast().declare(array.clone(), None).unwrap(),
            f.discard(f.values().address_of(place).unwrap()),
        ])];
        let facts = NumericFacts::check(&f.registry, &files).unwrap();
        let observation = facts.indices().next().unwrap();
        if lossy {
            assert_eq!(
                observation.nonwrapping_bounds(),
                Err(E::UnprovedSizeArithmetic)
            );
            let number = facts
                .analysis
                .obligations
                .iter()
                .find_map(|entry| {
                    if let Obligation::Index { index, .. } = &entry.kind {
                        Some(index)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(number.domain.integer_bounds(), Some((0, 255)));
            assert_eq!(number.losses.len(), 1);
            let Origin::Arithmetic(original) = number.losses[0] else {
                panic!("original loss")
            };
            assert!(matches!(
                original.kind(),
                CValueKind::Convert {
                    conversion: CConversion::Numeric(CScalarType::U8),
                    ..
                }
            ));
            assert_eq!(number.extent_bounds(), Err(E::UnprovedSizeArithmetic));
        } else {
            assert_eq!(observation.nonwrapping_bounds(), Ok((7, 7)));
        }
        let size = f
            .values()
            .numeric_conversion(CScalarType::Size, widened)
            .unwrap();
        let allocation = f
            .values()
            .call_value(f.values().known(CKnownCall::Allocate), vec![size])
            .unwrap();
        let checked = f.check(vec![
            f.ast().declare(array, None).unwrap(),
            f.discard(allocation),
        ]);
        if lossy {
            assert_eq!(checked, Err(E::UnprovedSizeArithmetic));
        } else {
            checked.unwrap();
        }
    }
}
