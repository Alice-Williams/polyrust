//! A public header/source pair traverses the same checked pipeline as one unit.
use super::{
    CDialect, CFileGrammar, CImportKind, CStructuralRenderer, package_fixture, project_c_package,
    resources,
};
use portable_codegen::{
    LinkedTargetPackage, OutputContents, TargetLinker, certify_resolved_package,
    render_certified_package, verify_unresolved_package,
};
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn linked(fixture: &package_fixture::Fixture) -> LinkedTargetPackage<CDialect> {
    let package = project_c_package(fixture.registry.clone(), fixture.files.clone()).unwrap();
    let checked = verify_unresolved_package(&CDialect, package).unwrap();
    TargetLinker::new(CDialect).link_ast(&checked).unwrap()
}

#[test]
fn public_pair_has_exact_primary_files_import_guard_and_definition_frames() {
    let fixture = package_fixture::fixture();
    let linked = linked(&fixture);
    let header = linked
        .files()
        .iter()
        .find(|file| file.module() == fixture.public.file())
        .unwrap();
    let source = linked
        .files()
        .iter()
        .find(|file| file.module() == fixture.helper.file())
        .unwrap();
    assert!(Arc::ptr_eq(
        &header.items()[0].unit.projection,
        &source.items()[0].unit.projection
    ));
    assert!(header.file_imports().is_empty());
    assert_eq!(source.file_imports().len(), 1);
    assert_eq!(source.file_imports()[0].destination(), header.file());
    let CFileGrammar::Header(guard) = header.source_kind() else {
        panic!("checked header guard")
    };
    let CImportKind::Generated(import) = source.file_imports()[0].kind() else {
        panic!("typed include")
    };
    assert_eq!(guard, import.guard());
    assert_eq!(guard.file(), header.module());
    assert!(header.items()[0].unit.data.bindings.values.is_empty());
    assert_eq!(header.items()[0].unit.data.declarations.len(), 1);
    assert_eq!(source.items()[0].unit.data.declarations.len(), 4);

    let measured = resources::measure_package(&linked).unwrap();
    assert!(measured.files[header.module()].function_frames.is_empty());
    assert_eq!(measured.files[header.module()].frame_bound, 0);
    assert_eq!(
        measured.files[source.module()].function_frames,
        measured.total.function_frames
    );
    assert_eq!(measured.total.function_frames.len(), 2);
    assert_eq!(
        measured.total.frame_bound,
        measured.total.function_frames.values().sum::<u64>()
    );
    assert_eq!(
        measured.total.nodes,
        measured.files.values().map(|file| file.nodes).sum::<u64>()
    );
    assert_eq!(
        measured.total.source_bound,
        measured
            .files
            .values()
            .map(|file| file.source_bound)
            .sum::<u64>()
    );
    // A partial registry/file view cannot obtain independent resource approval.
    assert!(resources::measure(&source.items()[0]).is_err());
    let guard_name = guard.identifier().as_str().to_owned();
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
    for _ in 0..2 {
        assert_eq!(
            output,
            render_certified_package(&CStructuralRenderer, &certified).unwrap()
        );
    }
    let texts: BTreeMap<_, _> = output
        .files()
        .iter()
        .map(|file| {
            let OutputContents::Text(text) = file.contents() else {
                panic!("C text")
            };
            (file.path(), text.as_str())
        })
        .collect();
    let header = texts["polyrust_crate_api.h"];
    let source = texts["crate_api.c"];
    assert!(header.starts_with(&format!("#ifndef {guard_name}\n#define {guard_name}\n")));
    assert!(header.ends_with(&format!("#endif /* {guard_name} */\n")));
    assert!(header.contains("int32_t poly_api(int32_t);"));
    assert!(!header.contains("poly_helper"));
    assert_eq!(
        source.matches("#include \"polyrust_crate_api.h\"").count(),
        1
    );
    assert!(source.contains("static int32_t poly_helper(int32_t);"));
    assert!(!source.contains("int32_t poly_api(int32_t);"));
    assert!(source.contains("int32_t poly_api(int32_t poly_input)"));
}

#[test]
fn caller_file_order_does_not_change_the_projected_package() {
    let mut fixture = package_fixture::fixture();
    let expected = project_c_package(fixture.registry.clone(), fixture.files.clone()).unwrap();
    fixture.files.reverse();
    assert_eq!(
        expected,
        project_c_package(fixture.registry, fixture.files).unwrap()
    );
}

#[test]
fn private_struct_and_member_bindings_never_appear_in_the_public_header() {
    let fixture = package_fixture::with_layout(package_fixture::RecordLayout::Implementation);
    let record = fixture.record.as_ref().unwrap();
    let linked = linked(&fixture);
    let header = linked
        .files()
        .iter()
        .find(|file| file.module() == fixture.public.file())
        .unwrap();
    let source = linked
        .files()
        .iter()
        .find(|file| file.module() == fixture.helper.file())
        .unwrap();
    assert!(header.items()[0].unit.data.bindings.types.is_empty());
    assert!(header.items()[0].unit.data.bindings.values.is_empty());
    let source_data = &source.items()[0].unit.data;
    assert!(source_data.bindings.types.contains_key(record));
    assert_eq!(
        source_data
            .bindings
            .values
            .keys()
            .filter(|value| matches!(value, super::bindings::CValueBinding::Member(_)))
            .count(),
        1
    );
    let name = source.items()[0].spelling.types[record].as_str().to_owned();
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("C text")
        };
        assert_eq!(
            text.contains(&format!("struct {name} {{")),
            file.path().ends_with(".c")
        );
        assert_eq!(text.contains("poly_secret"), file.path().ends_with(".c"));
    }
}

#[test]
fn public_header_owned_private_layout_is_rejected_even_with_authentic_registrations() {
    let fixture = package_fixture::with_layout(package_fixture::RecordLayout::Header);
    fixture
        .registry
        .registrations()
        .check_context(&fixture.files)
        .unwrap();
    let errors = project_c_package(fixture.registry, fixture.files).unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("only primary external scalar prototypes")
    }));
}

#[test]
fn independent_sibling_basenames_use_exact_registered_paths_not_name_inference() {
    let fixture = package_fixture::with_paths("polyrust_public_api.h", "private_implementation.c");
    let linked = linked(&fixture);
    let source = linked
        .files()
        .iter()
        .find(|file| file.module() == fixture.helper.file())
        .unwrap();
    let CImportKind::Generated(header) = source.file_imports()[0].kind() else {
        panic!("typed header")
    };
    assert_eq!(header.include_path(), "polyrust_public_api.h");
    assert_eq!(header.file(), fixture.public.file());
    certify_resolved_package(&CDialect, linked).unwrap();
}

#[test]
fn standard_header_shadowing_cannot_receive_a_certificate() {
    for path in [
        "stdint.h",
        "stddef.h",
        "STDINT.h",
        "assert.h",
        "features.h",
        "api.h",
        "polyrust_.h",
        "Polyrust_api.h",
    ] {
        let fixture = package_fixture::with_paths(path, "implementation.c");
        let result = project_c_package(fixture.registry, fixture.files)
            .and_then(|package| verify_unresolved_package(&CDialect, package))
            .and_then(|checked| TargetLinker::new(CDialect).link_ast(&checked))
            .and_then(|linked| certify_resolved_package(&CDialect, linked));
        assert!(
            result.is_err(),
            "shadowable standard header received a certificate: {path}"
        );
    }
}
