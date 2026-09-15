//! Owned constant objects traverse the normal shared proof and renderer.
use super::{
    CDialect, CStructuralRenderer,
    bindings::CValueBinding,
    owned_constant_fixture::{self, Shape},
    project_c_package, resources,
};
use crate::ast::{CConstness, CExpressions};
use portable_codegen::{
    LinkedTargetPackage, OutputContents, TargetLinker, certify_resolved_package,
    render_certified_package, verify_unresolved_package,
};

pub(super) fn linked(fixture: &owned_constant_fixture::Fixture) -> LinkedTargetPackage<CDialect> {
    let package = project_c_package(fixture.registry.clone(), fixture.files.clone()).unwrap();
    let checked = verify_unresolved_package(&CDialect, package).unwrap();
    TargetLinker::new(CDialect).link_ast(&checked).unwrap()
}

#[test]
fn constants_only_and_mixed_packages_use_typed_global_bindings() {
    for shape in [Shape::ConstantsOnly, Shape::Mixed] {
        let fixture = owned_constant_fixture::fixture(shape);
        let package = linked(&fixture);
        let measurements = resources::measure_package(&package).unwrap();
        assert_eq!(
            measurements.total.function_frames.len(),
            fixture.functions.len()
        );
        if shape == Shape::ConstantsOnly {
            assert_eq!(measurements.total.frame_bound, 0);
        }
        let header = package
            .files()
            .iter()
            .find(|file| file.module() == fixture.objects[0].file())
            .unwrap();
        assert_eq!(
            header.items()[0].unit.data.bindings.values.len(),
            fixture.objects.len()
        );
        let source = package
            .files()
            .iter()
            .find(|file| file.module() != fixture.objects[0].file())
            .unwrap();
        assert_eq!(source.file_imports().len(), 1);
        for object in &fixture.objects {
            let binding = CValueBinding::Global(object.clone());
            assert_eq!(
                header.items()[0].unit.data.bindings.values[&binding],
                source.items()[0].unit.data.bindings.values[&binding]
            );
            assert_eq!(
                header.items()[0].spelling.values[&binding],
                source.items()[0].spelling.values[&binding]
            );
            let expressions = CExpressions::new(fixture.registry.registrations());
            let place = expressions.global(object.clone()).unwrap();
            assert_eq!(place.ty().constness(), CConstness::Const);
            assert_eq!(
                expressions.read(place).unwrap().ty().constness(),
                CConstness::Unqualified
            );
        }
        let certificate = certify_resolved_package(&CDialect, package).unwrap();
        let rendered = render_certified_package(&CStructuralRenderer, &certificate).unwrap();
        assert_eq!(rendered.files().len(), 2);
        assert!(measurements.total.value_bytes >= 38);
        assert_eq!(measurements.total.automatic_objects, 0);
        assert!(
            rendered
                .files()
                .iter()
                .map(|file| match file.contents() {
                    OutputContents::Text(text) => text.len() as u64,
                    _ => unreachable!("C text"),
                })
                .sum::<u64>()
                <= measurements.total.source_bound
        );
        for output in rendered.files() {
            let OutputContents::Text(text) = output.contents() else {
                panic!("C text");
            };
            assert!(!text.contains("runtime"));
            for object in &fixture.objects {
                let comment = format!("Documentation for {}.", object.key().name.as_str());
                assert_eq!(
                    text.matches(&comment).count(),
                    usize::from(output.path().ends_with(".h"))
                );
            }
        }
        let mut reversed = fixture;
        reversed.files.reverse();
        let reversed = certify_resolved_package(&CDialect, linked(&reversed)).unwrap();
        assert_eq!(
            rendered,
            render_certified_package(&CStructuralRenderer, &reversed).unwrap()
        );
    }
}
