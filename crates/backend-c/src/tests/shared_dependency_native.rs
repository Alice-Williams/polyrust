//! Exact certificate output compiles as three separate owning C objects.
use super::{
    CDependencyApi, CDialect, CImportKind, CStructuralRenderer, dependency_fixture as fixture,
    resources,
};
use crate::ast::CScalarType;
use portable_codegen::{OutputContents, certify_resolved_package, render_certified_package};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "shared_dependency_native_reports.rs"]
mod reports;

struct Package {
    source: PathBuf,
    exports: Vec<String>,
    imports: BTreeSet<String>,
    frames: BTreeMap<String, u64>,
    bound: u64,
}

fn prepare(directory: &Path) -> Vec<Package> {
    let mut dependency: Option<CDependencyApi> = None;
    let mut packages = Vec::new();
    for crate_id in [10, 20, 30] {
        let functions = dependency
            .as_ref()
            .map(|api| api.functions().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let calls = if functions.is_empty() {
            [None, None]
        } else {
            [Some(0), Some(1)]
        };
        let input = fixture::fixture(
            crate_id,
            &[CScalarType::I32, CScalarType::Bool],
            &functions,
            &calls,
        );
        let linked = fixture::linked(&input);
        let measured = resources::measure_package(&linked).unwrap();
        let imports = linked
            .files()
            .iter()
            .flat_map(|file| file.imports())
            .filter(|import| matches!(import.kind(), CImportKind::Dependency(_)))
            .map(|import| import.binding().as_str().to_owned())
            .collect();
        let certified = certify_resolved_package(&CDialect, linked).unwrap();
        let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("C text")
            };
            let path = directory.join(file.path());
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        let api = CDependencyApi::from_certificate(certified).unwrap();
        let frames = super::c_defined_functions(api.package())
            .map(|definition| {
                (
                    definition.name().as_str().to_owned(),
                    measured.total.function_frames[definition.function()],
                )
            })
            .collect();
        packages.push(Package {
            source: directory.join(
                api.functions()
                    .next()
                    .unwrap()
                    .implementation()
                    .key()
                    .path
                    .as_str(),
            ),
            exports: api
                .functions()
                .map(|function| function.symbol().as_str().to_owned())
                .collect(),
            imports,
            frames,
            bound: measured.total.frame_bound,
        });
        dependency = Some(api);
    }
    packages
}

fn options(command: &mut Command, optimization: &str, sanitizer: Option<&str>) {
    command.args([
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
        "-fno-inline",
        "-fno-optimize-sibling-calls",
        optimization,
    ]);
    if let Some(kind) = sanitizer {
        command.args([
            format!("-fsanitize={kind}"),
            "-fno-sanitize-recover=all".into(),
            "-fno-pie".into(),
            "-no-pie".into(),
        ]);
    }
}

fn success(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(test)]
fn harness(package: &Package, order: usize) -> String {
    let headers = match order {
        0 => "#include \"polyrust_dep_30.h\"\n",
        1 => {
            "#include \"polyrust_dep_10.h\"\n#include \"polyrust_dep_20.h\"\n#include \"polyrust_dep_30.h\"\n"
        }
        _ => {
            "#include \"polyrust_dep_30.h\"\n#include \"polyrust_dep_20.h\"\n#include \"polyrust_dep_10.h\"\n"
        }
    };
    format!(
        "{headers}#include \"polyrust_dep_30.h\"\nint main(void) {{\n\
        const int32_t values[] = {{ INT32_MIN, INT32_MIN + 1, -1, 0, 1, INT32_MAX - 1, INT32_MAX }};\n\
        for (unsigned i = 0; i < sizeof(values)/sizeof(values[0]); ++i) {{\n\
        if ({}(values[i]) != values[i]) return 1; }}\n\
        uint32_t seed = UINT32_C(0x12345678);\n\
        for (unsigned i = 0; i < 8200; ++i) {{\n\
        seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5;\n\
        int32_t value = (int32_t)(seed & UINT32_C(0x7fffffff));\n\
        if (seed & UINT32_C(0x80000000)) value = -1 - value;\n\
        if ({}(value) != value) return 2; }}\n\
        if ({}(0) != 0 || {}(1) != 1) return 3;\nreturn 0;\n}}\n",
        package.exports[0], package.exports[0], package.exports[1], package.exports[1]
    )
}

#[test]
fn certified_dependency_chain_compiles_links_and_executes_as_separate_objects() {
    let directory =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-certified-dependencies");
    fs::create_dir_all(&directory).unwrap();
    let packages = prepare(&directory);
    let version = Command::new("gcc-14")
        .arg("-dumpfullversion")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(String::from_utf8(version.stdout).unwrap().trim(), "14.2.0");
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for (index, compiler, sanitizer) in [
        (0, PathBuf::from("gcc-14"), None),
        (1, zig, None),
        (2, PathBuf::from("gcc-14"), Some("address")),
        (3, PathBuf::from("gcc-14"), Some("undefined")),
    ] {
        for optimization in ["-O0", "-O2"] {
            let round = directory.join(format!("{index}{optimization}"));
            fs::create_dir_all(&round).unwrap();
            let mut objects = Vec::new();
            for (number, package) in packages.iter().enumerate() {
                let object = round.join(format!("package{number}.o"));
                let mut command = Command::new(&compiler);
                options(&mut command, optimization, sanitizer);
                command
                    .current_dir(&round)
                    .args(["-fstack-usage", "-I"])
                    .arg(&directory);
                if index == 1 {
                    command
                        .args(["-Xclang", "-stack-usage-file", "-Xclang"])
                        .arg(round.join(format!("package{number}.su")));
                }
                success(
                    command
                        .arg("-c")
                        .arg(&package.source)
                        .arg("-o")
                        .arg(&object),
                );
                reports::symbols(&object, package);
                objects.push(object);
            }
            reports::frames(&round, &packages);
            for order in 0..3 {
                let input = round.join(format!("consumer{order}.c"));
                fs::write(&input, harness(&packages[2], order)).unwrap();
                let object = round.join(format!("consumer{order}.o"));
                let mut command = Command::new(&compiler);
                options(&mut command, optimization, sanitizer);
                success(
                    command
                        .arg("-I")
                        .arg(&directory)
                        .arg("-c")
                        .arg(&input)
                        .arg("-o")
                        .arg(&object),
                );
                let binary = round.join(format!("consumer{order}"));
                let mut command = Command::new(&compiler);
                options(&mut command, optimization, sanitizer);
                success(command.args(&objects).arg(&object).arg("-o").arg(&binary));
                assert!(super::native_stack::run(&binary).success());
            }
        }
    }
}
