//! Public fields retain aggregate scope through allocation and certification.
use super::super::bindings::CValueBinding;
use super::*;
use std::{collections::BTreeSet, fs, path::PathBuf, process::Command};

fn package() -> LinkedTargetPackage<CDialect> {
    let (registry, files) = super::super::result_fixture::public_collision_fixture();
    let ast = project_c_package(registry, files).unwrap();
    let checked = verify_unresolved_package(&CDialect, ast).unwrap();
    TargetLinker::new(CDialect).link_ast(&checked).unwrap()
}

#[test]
fn public_fields_with_identical_names_use_distinct_typed_owners() {
    let linked = package();
    let header = linked
        .files()
        .iter()
        .find(|f| f.module().key().path.as_str().ends_with(".h"))
        .unwrap();
    let unit = &header.items()[0];
    let mut owners = BTreeSet::new();
    for (value, spelling) in &unit.spelling.values {
        let CValueBinding::Member(member) = value else {
            continue;
        };
        assert_eq!(
            spelling.as_str(),
            format!("poly_{}", member.key().name.as_str())
        );
        owners.insert(member.owner().clone());
    }
    assert_eq!(owners.len(), 2);
    let function = unit
        .spelling
        .functions
        .iter()
        .find(|(f, _)| f.key().name.as_str() == "entry")
        .unwrap()
        .1;
    assert_eq!(function.as_str(), "poly_entry");
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    render_certified_package(&CStructuralRenderer, &certified).unwrap();
}

#[test]
fn repeated_public_fields_and_function_name_collisions_compile_natively() {
    let certified = certify_resolved_package(&CDialect, package()).unwrap();
    let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("public-result-scopes");
    fs::create_dir_all(&root).unwrap();
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        fs::write(root.join(file.path()), text).unwrap();
    }
    fs::write(root.join("consumer.c"), consumer()).unwrap();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for compiler in [&PathBuf::from("gcc-14"), &zig] {
        for opt in ["-O0", "-O2"] {
            for source in ["result", "consumer"] {
                native::compile(
                    compiler,
                    &root.join(format!("{source}.c")),
                    &root.join(format!("{source}.o")),
                    opt,
                    true,
                    false,
                );
            }
            let binary = root.join("test");
            native::run(
                Command::new(compiler)
                    .args(["-fsanitize=undefined", "-fno-sanitize-recover=all"])
                    .arg(root.join("result.o"))
                    .arg(root.join("consumer.o"))
                    .arg("-o")
                    .arg(&binary),
            );
            native::run(Command::new(binary).env("UBSAN_OPTIONS", "halt_on_error=1"));
        }
    }
}

#[cfg(test)]
fn consumer() -> &'static str {
    "#include \"polyrust_result.h\"\n\
     int main(void) {\n\
     struct poly_result first = poly_construct(1, INT32_MIN);\n\
     struct poly_other_result second = { .poly_success = 0, .poly_entry = INT32_MAX };\n\
     if (!first.poly_success || first.poly_entry != INT32_MIN) return 1;\n\
     if (second.poly_success || second.poly_entry != INT32_MAX) return 2;\n\
     if (poly_entry(1, first.poly_entry, 0) != INT32_MIN) return 3;\n\
     return 0; }\n"
}
