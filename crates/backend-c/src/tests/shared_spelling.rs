//! Native oracle executes the bytes returned by the shared certified renderer.
use super::{CDialect, project_c_package, spelling, tests::fixture};
use crate::ast::{CBinaryOperator, CConstness, CScalarType};
use portable_codegen::{TargetLinker, verify_unresolved_package};
use std::{fs, process::Command};

fn source(scalar: CScalarType) -> String {
    linked_source(fixture(scalar))
}

pub(super) fn linked_source(
    (registry, source): (crate::ast::CFrozenRegistry, crate::ast::CSourceFile),
) -> String {
    generated((registry, source)).0
}

fn generated(
    (registry, source): (crate::ast::CFrozenRegistry, crate::ast::CSourceFile),
) -> (String, super::resources::Measurements) {
    let package = project_c_package(registry, vec![source]).unwrap();
    let verified = verify_unresolved_package(&CDialect, package).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&verified).unwrap();
    let file = &linked.files()[0];
    let first = spelling::file(file);
    let measured = super::resources::measure(&file.items()[0]).unwrap();
    assert!(measured.source_bound >= first.len() as u64);
    assert!(measured.nodes > 0);
    assert!(measured.automatic_bytes > 0);
    assert_eq!(first, spelling::file(file));
    assert_eq!(first, spelling::file(file));
    let certified = portable_codegen::certify_resolved_package(&super::CDialect, linked).unwrap();
    let rendered =
        portable_codegen::render_certified_package(&super::CStructuralRenderer, &certified)
            .unwrap();
    assert_eq!(rendered.files().len(), 1);
    let portable_codegen::OutputContents::Text(text) = rendered.files()[0].contents() else {
        panic!("expected C source");
    };
    assert_eq!(text, &format!("{}\n", first.trim_end()));
    for _ in 0..2 {
        assert_eq!(
            rendered,
            portable_codegen::render_certified_package(&super::CStructuralRenderer, &certified)
                .unwrap()
        );
    }
    (text.clone(), measured)
}

#[test]
fn scalar_spelling_has_exact_includes_names_and_prototypes() {
    let prefix = format!(
        "#include <stdint.h>\n#include <stddef.h>\n\n{}",
        super::platform_tests::expected()
    );
    assert_eq!(
        source(CScalarType::I32),
        format!(
            "{prefix}{}",
            concat!(
                "int32_t poly_identity(int32_t);\n\n",
                "int32_t poly_identity(int32_t poly_input) {\n",
                "    return poly_input;\n}\n",
            )
        )
    );
    assert_eq!(
        source(CScalarType::Bool),
        format!(
            "{prefix}{}",
            concat!(
                "_Bool poly_identity(_Bool);\n\n",
                "_Bool poly_identity(_Bool poly_input) {\n",
                "    return poly_input;\n}\n",
            )
        )
    );
}

#[test]
fn definition_keeps_parameter_local_constness_without_changing_prototype() {
    let source = linked_source(super::tests::qualified_fixture(
        CScalarType::I32,
        CConstness::Const,
    ));
    assert!(source.contains("int32_t poly_identity(int32_t);"));
    assert!(source.contains("int32_t poly_identity(const int32_t poly_input)"));
}

#[test]
fn generated_scalar_units_compile_and_execute_with_strict_pinned_gcc() {
    let version = Command::new("gcc-14")
        .arg("-dumpfullversion")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(String::from_utf8(version.stdout).unwrap().trim(), "14.2.0");
    let directory = std::path::PathBuf::from(
        std::env::var_os("TEST_TMPDIR").expect("Bazel native oracle environment"),
    )
    .join("c-shared-spelling");
    fs::create_dir_all(&directory).unwrap();
    let mut sources = vec![
        generated(fixture(CScalarType::I32)),
        generated(fixture(CScalarType::Int)),
        generated(fixture(CScalarType::Bool)),
        generated(super::record_fixture::fixture()),
        generated(super::tests::qualified_fixture(
            CScalarType::I32,
            CConstness::Const,
        )),
    ]
    .into_iter()
    .map(|source| (source, vec![("0".to_owned(), 0), ("1".to_owned(), 1)]))
    .collect::<Vec<_>>();
    for operator in [
        CBinaryOperator::Equal,
        CBinaryOperator::NotEqual,
        CBinaryOperator::Less,
        CBinaryOperator::LessEqual,
        CBinaryOperator::Greater,
        CBinaryOperator::GreaterEqual,
    ] {
        let generated = generated(super::record_fixture::comparison_fixture(operator));
        let expectations = [i32::MIN, -12345, -1, 0, 1, 42, i32::MAX]
            .into_iter()
            .map(|value| {
                let selected = match operator {
                    CBinaryOperator::Equal => value == 0,
                    CBinaryOperator::NotEqual => value != 0,
                    CBinaryOperator::Less => value < 0,
                    CBinaryOperator::LessEqual => value <= 0,
                    CBinaryOperator::Greater => value > 0,
                    CBinaryOperator::GreaterEqual => value >= 0,
                    _ => unreachable!("comparison fixture inventory"),
                };
                (value.to_string(), if selected { value } else { i32::MIN })
            })
            .collect();
        sources.push((generated, expectations));
    }
    use super::capacity_fixture::{Shape, fixture as capacity_fixture};
    for shape in [
        Shape::Parameters(127),
        Shape::Fields(256),
        Shape::NestedBlocks(45),
        Shape::IdentifierBytes(256),
        Shape::CommentBytes(1024 * 1024),
        Shape::Combined,
        Shape::Conversions(91),
    ] {
        let fixture = capacity_fixture(shape);
        let generated = generated(fixture);
        let arguments = match shape {
            Shape::Parameters(count) => vec!["42"; count].join(", "),
            Shape::Combined => vec!["42"; 127].join(", "),
            _ => "42".into(),
        };
        sources.push((generated, vec![(arguments, 42)]));
    }
    for (locals, fields) in [(499, 0), (27, 256)] {
        sources.push((
            generated(super::storage_fixture::fixture(locals, fields)),
            vec![("42".into(), 42)],
        ));
    }
    for (index, ((generated, measured), expectations)) in sources.into_iter().enumerate() {
        let input = directory.join(format!("unit_{index}.c"));
        let cases = expectations
            .into_iter()
            .map(|(value, expected)| format!("if (poly_identity({value}) != {expected}) return 1;"))
            .collect::<Vec<_>>()
            .join("\n");
        let consumer = format!("\nint main(void) {{\n{cases}\nreturn 0;\n}}\n");
        fs::write(&input, format!("{generated}{consumer}")).unwrap();
        let zig = std::path::PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
            .join(std::env::var_os("TEST_WORKSPACE").unwrap())
            .join("tools/c/zig_native_oracle");
        for (compiler_index, (compiler, instrumentation)) in [
            (std::path::PathBuf::from("gcc-14"), None),
            (zig, None),
            (
                std::path::PathBuf::from("gcc-14"),
                Some("-fsanitize=address"),
            ),
            (
                std::path::PathBuf::from("gcc-14"),
                Some("-fsanitize=undefined"),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            for optimization in ["-O0", "-O2"] {
                let round = directory.join(format!("unit_{index}_{compiler_index}{optimization}"));
                fs::create_dir_all(&round).unwrap();
                let binary = round.join("probe");
                let object = round.join("probe.o");
                let mut command = Command::new(&compiler);
                command.current_dir(&round);
                if compiler_index == 1 {
                    // Zig's driver does not publish the implicit Clang sidecar.
                    // Give the frontend an explicit output in this test round.
                    command.args(["-Xclang", "-stack-usage-file", "-Xclang"]);
                    command.arg(round.join("probe.su"));
                }
                if let Some(flag) = instrumentation {
                    command.args([flag, "-fno-sanitize-recover=all", "-fno-pie", "-no-pie"]);
                }
                let output = command
                    .args([
                        "-std=c17",
                        "-Wall",
                        "-Wextra",
                        "-Wpedantic",
                        "-Werror",
                        "-Wstrict-prototypes",
                        "-Wmissing-prototypes",
                        "-fno-fast-math",
                        "-ffp-contract=off",
                        "-fsigned-char",
                        "-fno-short-enums",
                        "-fstack-usage",
                        optimization,
                    ])
                    .arg("-c")
                    .arg(&input)
                    .arg("-o")
                    .arg(&object)
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                super::frame_reports::check(&round, measured.frame_bound);
                let mut link = Command::new(&compiler);
                if let Some(flag) = instrumentation {
                    link.args([flag, "-fno-pie", "-no-pie"]);
                }
                let output = link.arg(&object).arg("-o").arg(&binary).output().unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(super::native_stack::run(&binary).success());
            }
        }
    }
}
