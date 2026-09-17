//! Guarded-stack and watermark evidence for the platform library allowance.
use super::*;
use std::{fs, path::PathBuf, process::Command};

fn run(command: &mut Command) -> std::process::Output {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn truncation_native_library_stack_has_measured_nonzero_headroom() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-truncation-stack");
    fs::create_dir_all(&root).unwrap();
    let owners = chain();
    let members: Vec<_> = owners
        .iter()
        .map(|api| api.functions().next().unwrap().symbol().as_str())
        .collect();
    for api in &owners {
        let output = render_certified_package(&CStructuralRenderer, api.package()).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            fs::write(root.join(file.path()), text).unwrap();
        }
    }
    let cases = native::cases()
        .into_iter()
        .map(|bits| format!("{{UINT64_C({bits}),UINT64_C({})}}", native::expected(bits)))
        .collect::<Vec<_>>()
        .join(",\n");
    let probe = include_str!("../../test/trunc_stack_probe.c")
        .replace("PROBE_FUNCTION", members[2])
        .replace("PROBE_CASES", &cases);
    fs::write(root.join("probe.c"), probe).unwrap();
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
            let directory = root.join(format!("{index}{optimization}"));
            fs::create_dir_all(&directory).unwrap();
            let mut objects = vec![];
            let mut observed = std::collections::BTreeMap::new();
            for stem in [
                "polyrust_dep_101",
                "polyrust_dep_102",
                "polyrust_dep_103",
                "probe",
            ] {
                let object = directory.join(format!("{stem}.o"));
                let report = directory.join(format!("{stem}.su"));
                let mut compile = Command::new(&compiler);
                compile.args([
                    "-std=c17",
                    "-Wall",
                    "-Wextra",
                    "-Wpedantic",
                    "-Werror",
                    "-Wstrict-prototypes",
                    "-Wmissing-prototypes",
                    "-fno-fast-math",
                    "-ffp-contract=off",
                    "-fno-builtin",
                    "-fno-inline",
                    "-fno-optimize-sibling-calls",
                    "-fstack-usage",
                    optimization,
                    "-pthread",
                ]);
                if index == 1 {
                    compile
                        .args(["-Xclang", "-stack-usage-file", "-Xclang"])
                        .arg(&report);
                }
                if let Some(kind) = sanitizer {
                    compile.args([
                        format!("-fsanitize={kind}"),
                        "-fno-sanitize-recover=all".into(),
                        "-fno-pie".into(),
                        "-no-pie".into(),
                    ]);
                }
                run(compile
                    .arg("-c")
                    .arg(root.join(format!("{stem}.c")))
                    .arg("-o")
                    .arg(&object));
                if stem != "probe" {
                    for line in fs::read_to_string(&report).unwrap().lines() {
                        let fields: Vec<_> = line.split('\t').collect();
                        let name = fields[0].rsplit(':').next().unwrap();
                        if members.contains(&name) {
                            assert!(matches!(fields[2], "static" | "dynamic,bounded"));
                            assert!(
                                observed
                                    .insert(name.to_owned(), fields[1].parse::<u64>().unwrap())
                                    .is_none()
                            );
                        } else {
                            assert!(
                                matches!(
                                    name,
                                    "_sub_I_00099_0"
                                        | "_sub_I_00099_1"
                                        | "_sub_D_00099_0"
                                        | "_sub_D_00099_1"
                                ),
                                "unknown generated frame {name}"
                            );
                        }
                    }
                }
                objects.push(object);
            }
            assert_eq!(
                observed.len(),
                members.len(),
                "all generated frames need reports"
            );
            for (api, member) in owners.iter().zip(&members) {
                assert!(observed[*member] <= api.functions().next().unwrap().stack_bound_bytes());
            }
            let binary = directory.join("probe");
            let mut link = Command::new(&compiler);
            link.args(objects).arg("-pthread");
            for library in owners[2].system_libraries() {
                match library {
                    CSystemLibrary::Math => {
                        link.arg("-lm");
                    }
                }
            }
            if let Some(kind) = sanitizer {
                link.args([
                    format!("-fsanitize={kind}"),
                    "-fno-pie".into(),
                    "-no-pie".into(),
                ]);
            }
            run(link.arg("-o").arg(&binary));
            let invoke = |size: u64, allowance: u64, guard: Option<&str>| {
                let mut command = Command::new(&binary);
                command
                    .args([size.to_string(), allowance.to_string()])
                    .env("ASAN_OPTIONS", "detect_leaks=1:halt_on_error=1")
                    .env("UBSAN_OPTIONS", "halt_on_error=1");
                if let Some(guard) = guard {
                    command.arg(guard);
                }
                command.output().unwrap()
            };
            for size in [65536, 262144] {
                let output = invoke(size, 65536, None);
                assert!(
                    output.status.success(),
                    "{index} {optimization}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let used = String::from_utf8(output.stdout)
                    .unwrap()
                    .trim()
                    .parse::<u64>()
                    .unwrap();
                assert!(used > 1 && used <= 65536);
                eprintln!(
                    "trunc stack {index} {optimization} arena={size}: watermark={used}, generated={observed:?}"
                );
            }
            let underestimated = invoke(262144, 1, None);
            assert_eq!(
                underestimated.status.code(),
                Some(3),
                "a one-byte allowance must fail"
            );
            for guard in ["lower", "upper"] {
                assert!(
                    !invoke(262144, 65536, Some(guard)).status.success(),
                    "ineffective {guard} guard"
                );
            }
        }
    }
}
