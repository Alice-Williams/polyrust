//! Resource estimates include typed syntax and fail closed on arithmetic drift.
use super::super::{CFileGrammar, package_fixture, package_projection_tests::linked};
use super::{Measurements, measure_package, measure_units, syntax, totals};

#[test]
fn header_and_include_bytes_are_charged_and_underestimates_fail_closed() {
    let fixture = package_fixture::fixture();
    let linked = linked(&fixture);
    let units: Vec<_> = linked
        .files()
        .iter()
        .flat_map(|file| file.items())
        .collect();
    let plain = measure_units(&units).unwrap();
    let measured = measure_package(&linked).unwrap();
    for file in linked.files() {
        let before = &plain.files[file.module()];
        let after = &measured.files[file.module()];
        assert!(after.source_bound > before.source_bound);
        assert_eq!(before.function_frames, after.function_frames);
        if let CFileGrammar::Header(guard) = file.source_kind() {
            let bytes = guard.identifier().as_str().len();
            assert!(after.max_identifier_bytes >= bytes);
            assert!(after.source_bound - before.source_bound >= bytes as u64 * 3 + 64);
        }
    }
    syntax::verify_output(&linked, &measured).unwrap();
    let mut too_small = measured.clone();
    too_small
        .files
        .get_mut(fixture.public.file())
        .unwrap()
        .source_bound = 0;
    assert!(
        syntax::verify_output(&linked, &too_small)
            .unwrap_err()
            .contains("formatted file")
    );
    let mut too_small = measured;
    too_small.total.source_bound = 0;
    assert!(
        syntax::verify_output(&linked, &too_small)
            .unwrap_err()
            .contains("formatted package")
    );
}

#[derive(Clone, Copy, Debug)]
enum Additive {
    Nodes,
    Comments,
    Diagnostics,
    AutomaticBytes,
    AutomaticObjects,
    Values,
    Source,
}

fn set(measured: &mut Measurements, field: Additive, value: u64) {
    match field {
        Additive::Nodes => measured.nodes = value,
        Additive::Comments => measured.comment_bytes = value,
        Additive::Diagnostics => measured.diagnostic_bytes = value,
        Additive::AutomaticBytes => measured.automatic_bytes = value,
        Additive::AutomaticObjects => measured.automatic_objects = value,
        Additive::Values => measured.value_bytes = value,
        Additive::Source => measured.source_bound = value,
    }
}

#[test]
fn every_additive_package_dimension_checks_overflow() {
    for field in [
        Additive::Nodes,
        Additive::Comments,
        Additive::Diagnostics,
        Additive::AutomaticBytes,
        Additive::AutomaticObjects,
        Additive::Values,
        Additive::Source,
    ] {
        let mut largest = Measurements::default();
        set(&mut largest, field, u64::MAX);
        let mut next = Measurements::default();
        assert!(totals::sum([&largest, &next]).is_ok());
        set(&mut next, field, 1);
        assert!(totals::sum([&largest, &next]).is_err(), "{field:?}");
    }
    assert!(
        totals::source_bound(&Measurements {
            nodes: u64::MAX,
            ..Measurements::default()
        })
        .is_err()
    );
    assert!(
        totals::source_bound(&Measurements {
            diagnostic_bytes: u64::MAX,
            ..Measurements::default()
        })
        .is_err()
    );
}
