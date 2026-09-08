use std::collections::BTreeSet;

use crate::generator::Generator;
use crate::{
    CBackend, CCode, CImport, CImportKind, CRenderer, c_runtime_header_file, c_runtime_source_file,
};
use portable_check::v0::CheckedProgram;
use portable_codegen::{
    Backend, BackendOptions, DeclaredDependency, FileGroup, FileGroupId, LanguageFile,
    LanguagePackage, LanguageSourceFile, OutputContents, OutputManifest,
};
use portable_ir::v0::TypeRef;

#[test]
fn descriptor_and_manifest_are_deterministic() {
    let checked = fixture();
    assert_eq!(CBackend.descriptor().target.as_str(), "org.polyrust.c");
    let first = CBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let second = CBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let third = CBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    assert_eq!(first.canonical_json(), second.canonical_json());
    assert_eq!(second.canonical_json(), third.canonical_json());
    assert!(first.dependencies().is_empty());
    assert!(first.file("src/generated.c").is_some());
}

#[test]
fn aggregate_abi_is_concrete_and_deterministic() {
    let checked = portable_check::v0::check_program(
        portable_ir::v0::from_json(include_bytes!("../../test/abi-shapes.poly.json")).unwrap(),
    )
    .unwrap();
    let first = CBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let second = CBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    assert_eq!(first.canonical_json(), second.canonical_json());
    let header = match first.file("src/generated.h").unwrap().contents() {
        OutputContents::Text(text) => text,
        OutputContents::Bytes(_) => panic!("generated C header must be text"),
    };
    assert!(header.contains("struct abi_shapes_list__named_1"));
    assert!(header.contains("struct abi_shapes_option__named_1"));
    assert!(header.contains("struct abi_shapes_result__option__string__bytes"));
    assert!(header.contains("typedef enum abi_shapes_Choice_tag"));
    assert!(!header.contains("void *"));
}

#[test]
fn c_includes_and_abi_types_are_validated_fragments() {
    for header in ["stdbool.h", "sys/types.h", "generated.h"] {
        let result = if header == "generated.h" {
            CImport::local(header)
        } else {
            CImport::system(header)
        };
        assert!(result.is_ok(), "{header}");
    }
    for header in [
        "",
        "../escape.h",
        "/absolute.h",
        "bad\\path.h",
        "x.h>\n#include <y",
    ] {
        assert!(CImport::system(header).is_err(), "{header}");
        assert!(CImport::local(header).is_err(), "{header}");
    }
    assert!(CImport::local("not_a_header.hpp").is_err());

    let program = fixture();
    let generator = Generator::new(&program);
    for (ty, expected_text, expected_headers) in [
        (TypeRef::Unit, "registration_unit", &[][..]),
        (TypeRef::Bool, "bool", &["stdbool.h"][..]),
        (TypeRef::I64, "int64_t", &["stdint.h"][..]),
        (TypeRef::String, "poly_string", &[][..]),
    ] {
        let code = generator.ty(&ty);
        assert_eq!(code.text, expected_text);
        assert_eq!(system_headers(&code), string_set(expected_headers));
    }
    for (ty, expected_headers) in [
        (
            TypeRef::List(Box::new(TypeRef::I64)),
            &["stdbool.h", "stddef.h", "stdint.h"][..],
        ),
        (
            TypeRef::Option(Box::new(TypeRef::String)),
            &["stdbool.h"][..],
        ),
        (
            TypeRef::Result {
                ok: Box::new(TypeRef::Bytes),
                error: Box::new(TypeRef::String),
            },
            &["stdbool.h"][..],
        ),
    ] {
        let code = generator.composite_shape_header(&ty);
        assert_eq!(system_headers(&code), string_set(expected_headers));
        assert!(code.helper_roots.contains("runtime.core"));
    }
}

#[test]
fn c_runtime_helper_matrix_is_exact_and_minimal() {
    let core = string_set(&["runtime.core"]);
    let header = render_runtime_header(&core);
    assert_eq!(
        include_headers(&header),
        string_set(&["stdbool.h", "stddef.h", "stdint.h"])
    );
    assert!(!header.contains("poly_f64_trunc"));
    assert!(!header.contains("poly_f64_is_negative_zero"));
    assert!(!header.contains("poly_f64_abs"));
    assert!(!header.contains("poly_string_replace_all"));
    let source = render_runtime_source(&core);
    assert_eq!(
        include_headers(&source),
        string_set(&["stdlib.h", "string.h"])
    );
    assert_eq!(source.matches("#include \"runtime.h\"").count(), 1);
    assert!(!source.contains("POLYRUST-"));
    assert!(!source.contains("poly_f64_trunc"));
    assert!(!source.contains("poly_f64_is_negative_zero"));
    assert!(!source.contains("poly_f64_abs"));

    for (root, present, absent) in [
        (
            "runtime.feature.f64",
            "poly_f64_trunc",
            "poly_string_replace_all",
        ),
        (
            "runtime.feature.string-replace-all",
            "poly_string_replace_all",
            "poly_bytes_replace_all",
        ),
        (
            "runtime.feature.bytes-replace-all",
            "poly_bytes_replace_all",
            "poly_string_replace_many",
        ),
        (
            "runtime.feature.string-replace-many",
            "poly_string_replace_many",
            "poly_string_truncate_utf8_bytes",
        ),
        (
            "runtime.feature.string-truncate-utf8",
            "poly_string_truncate_utf8_bytes",
            "poly_string_trim_start",
        ),
        (
            "runtime.feature.string-trim",
            "poly_string_trim_start",
            "poly_string_replace_all",
        ),
    ] {
        let roots = string_set(&[root]);
        let rendered = render_runtime_source(&roots);
        assert!(rendered.contains(present), "{root} lacks {present}");
        assert!(!rendered.contains(absent), "{root} includes {absent}");
        assert!(!rendered.contains("POLYRUST-"));
        if root == "runtime.feature.f64" {
            assert!(rendered.contains("#include <math.h>"));
            assert!(rendered.contains("poly_f64_is_negative_zero"));
            assert!(rendered.contains("poly_f64_abs"));
        } else {
            assert!(!rendered.contains("#include <math.h>"));
        }
    }

    let manifest = CBackend
        .generate(&fixture(), &BackendOptions::default())
        .unwrap();
    let minimal = generated_text(&manifest, "src/runtime.c");
    assert!(!minimal.contains("poly_string_replace_all"));
    assert!(!minimal.contains("poly_f64_trunc"));
    assert!(!minimal.contains("poly_f64_is_negative_zero"));
    assert!(!minimal.contains("poly_f64_abs"));
    assert!(!minimal.contains("#include <math.h>"));
}

#[test]
fn c_includes_and_guards_are_owned_per_language_file() {
    let manifest = CBackend
        .generate(&fixture(), &BackendOptions::default())
        .unwrap();
    let runtime_header = generated_text(&manifest, "src/runtime.h");
    assert_eq!(runtime_header.matches("#include <stdbool.h>").count(), 1);
    assert!(!runtime_header.contains("#include <stdlib.h>"));
    assert_eq!(
        runtime_header.matches("#ifndef POLYRUST_RUNTIME_H").count(),
        1
    );
    assert_eq!(
        runtime_header
            .matches("#endif /* POLYRUST_RUNTIME_H */")
            .count(),
        1
    );

    let runtime_source = generated_text(&manifest, "src/runtime.c");
    assert_eq!(runtime_source.matches("#include \"runtime.h\"").count(), 1);
    assert_eq!(runtime_source.matches("#include <stdlib.h>").count(), 1);
    let generated_header = generated_text(&manifest, "src/generated.h");
    assert_eq!(
        generated_header.matches("#include \"runtime.h\"").count(),
        1
    );
    assert!(!generated_header.contains("#include <string.h>"));
    let generated_source = generated_text(&manifest, "src/generated.c");
    assert_eq!(
        generated_source.matches("#include \"generated.h\"").count(),
        1
    );
    assert!(!generated_source.contains("#include <string.h>"));
    let conformance = generated_text(&manifest, "tests/conformance_test.c");
    assert_eq!(conformance.matches("#include <limits.h>").count(), 1);
    assert!(!conformance.contains("#include <string.h>"));
}

fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
    match manifest.file(path).unwrap().contents() {
        OutputContents::Text(text) => text,
        OutputContents::Bytes(_) => panic!("C source must be text"),
    }
}

fn system_headers(code: &CCode) -> BTreeSet<String> {
    code.imports
        .iter()
        .filter_map(|(_, import)| match &import.kind {
            CImportKind::System { path } => Some(path.clone()),
            CImportKind::Local { .. } => None,
        })
        .collect()
}

fn string_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[cfg(test)]
fn include_headers(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter_map(|line| line.strip_prefix("#include <")?.strip_suffix('>'))
        .map(str::to_owned)
        .collect()
}

fn render_runtime_header(roots: &BTreeSet<String>) -> String {
    render_source_file(c_runtime_header_file(roots).unwrap(), "src/runtime.h")
}

fn render_runtime_source(roots: &BTreeSet<String>) -> String {
    render_source_file(c_runtime_source_file(roots).unwrap(), "src/runtime.c")
}

fn render_source_file(file: LanguageSourceFile<CImport>, path: &str) -> String {
    let group = FileGroup::new(
        FileGroupId::parse("test").unwrap(),
        vec![LanguageFile::source(file)],
    )
    .unwrap();
    let package =
        LanguagePackage::new(vec![group], Vec::<DeclaredDependency>::new(), Vec::new()).unwrap();
    let manifest = portable_codegen::render_language_package(&package, &CRenderer).unwrap();
    generated_text(&manifest, path).to_owned()
}

fn fixture() -> CheckedProgram {
    portable_check::v0::check_program(
        portable_ir::v0::from_json(include_bytes!(
            "../../../build/testdata/registration.poly.json"
        ))
        .unwrap(),
    )
    .unwrap()
}
