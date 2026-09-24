//! Native oracle only: no error inspection is admitted into translated source.
#![forbid(unsafe_code)]
use portable_codegen::RustIntegerErrorKind as Kind;
use std::num::{NonZeroI32, TryFromIntError};

fn errors() -> [(Kind, TryFromIntError); 3] {
    [
        (Kind::PosOverflow, i32::try_from(i64::MAX).unwrap_err()),
        (Kind::NegOverflow, i32::try_from(i64::MIN).unwrap_err()),
        (Kind::Zero, NonZeroI32::try_from(0).unwrap_err()),
    ]
}

fn classify(error: TryFromIntError) -> Kind {
    errors()
        .into_iter()
        .find(|(_, original)| *original == error)
        .unwrap()
        .0
}

#[allow(clippy::needless_match)] // Exercise explicit Err binding/reconstruction.
fn reconstruct(value: Result<i32, TryFromIntError>) -> Result<i32, TryFromIntError> {
    match value {
        Ok(value) => Ok(value),
        Err(error) => Err(error),
    }
}

#[test]
fn pinned_error_is_stateful_and_all_public_driver_kinds_are_distinct() {
    assert_eq!(std::mem::size_of::<TryFromIntError>(), 1);
    let values = errors();
    for (index, (kind, error)) in values.into_iter().enumerate() {
        assert_eq!(classify(error), kind);
        // Independent pinned std implementation observation: exchanging the
        // labels in errors() must not preserve this oracle. Debug is test-only,
        // not a stable wire protocol or an admitted source operation.
        let observed = format!("{error:?}");
        let expected = match kind {
            Kind::PosOverflow => "TryFromIntError(PosOverflow)",
            Kind::NegOverflow => "TryFromIntError(NegOverflow)",
            Kind::Zero => "TryFromIntError(Zero)",
            _ => unreachable!("native driver has three public constructors"),
        };
        assert_eq!(observed, expected);
        for (_, other) in &values[index + 1..] {
            assert_ne!(error, *other);
        }
    }
}

#[test]
fn copying_forwarding_and_reconstruction_preserve_each_reachable_kind() {
    let mut rows = 0;
    for (kind, error) in errors() {
        for depth in 0..16 {
            let original = Err(error);
            let mut value = original;
            for _ in 0..depth {
                value = reconstruct(value);
            }
            assert_eq!(value, original);
            assert_eq!(classify(value.unwrap_err()), kind);
            rows += 1;
        }
    }
    for success in [i32::MIN, -1, 0, 1, i32::MAX] {
        assert_eq!(reconstruct(Ok(success)), Ok(success));
    }
    assert_eq!(rows, 48);
}

#[test]
fn compiling_collapse_and_swap_controls_are_detected_by_value_not_tag() {
    let [(positive_kind, positive), (_, negative), (_, zero)] = errors();
    let mut collapse_failures = 0;
    let mut swap_failures = 0;
    for (kind, original) in errors() {
        let collapsed: Result<i32, _> = Err(positive);
        let swapped: Result<i32, _> = Err(match kind {
            Kind::PosOverflow => negative,
            Kind::NegOverflow => positive,
            Kind::Zero => zero,
            _ => unreachable!("native driver has exactly three public constructors"),
        });
        assert!(collapsed.is_err() && swapped.is_err());
        assert_eq!(classify(positive), positive_kind);
        collapse_failures += usize::from(collapsed != Err(original));
        swap_failures += usize::from(swapped != Err(original));
    }
    assert_eq!((collapse_failures, swap_failures), (2, 2));
}
