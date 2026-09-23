//! Strict compiled proof of actual rendered struct transport and frame bounds.
use super::*;
use std::{fs, path::PathBuf, process::Command};

fn probe(mode: Mutation) -> super::super::call_native_tests::Probe {
    let linked = linked(mode).unwrap();
    let unit = &linked.files()[0].items()[0];
    let measured = super::super::resources::measure(unit).unwrap();
    let names: Vec<_> = ["entry", "construct", "forward", "inspect"]
        .iter()
        .map(|key| {
            unit.spelling
                .functions
                .iter()
                .find(|(function, _)| function.key().name.as_str() == *key)
                .unwrap()
                .1
                .as_str()
                .to_owned()
        })
        .collect();
    let ty = unit.spelling.types.values().next().unwrap().as_str();
    // Test-only exact function pointer types retain helper definitions at O2.
    let keepalive = format!(
        "struct {ty} (*keep_construct)(_Bool, int32_t) = {};\n\
         struct {ty} (*keep_forward)(struct {ty}) = {};\n\
         int32_t (*keep_inspect)(struct {ty}, _Bool) = {};",
        names[1], names[2], names[3],
    );
    let frames = measured
        .function_frames
        .iter()
        .map(|(f, bound)| (unit.spelling.functions[f].as_str().to_owned(), *bound))
        .collect();
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("text")
    };
    super::super::call_native_tests::Probe {
        text: text.clone(),
        frames,
        bound: measured.frame_bound,
        names,
        keepalive,
    }
}

#[test]
fn private_result_native_values_faults_and_frames() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("private-results");
    fs::create_dir_all(&root).unwrap();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for (case, mode) in [Mutation::None, Mutation::SwappedTag, Mutation::LostPayload]
        .into_iter()
        .enumerate()
    {
        let probe = probe(mode);
        let input = root.join(format!("result{case}.c"));
        fs::write(
            &input,
            format!(
                "{}\n{}\nint main(void) {{\n\
             const int32_t values[] = {{INT32_MIN, -1, 0, 1, 17, INT32_MAX}};\n\
             for (unsigned int t = 0; t < 2; ++t) {{\n\
             for (unsigned int i = 0; i < 65542; ++i) {{\n\
             int32_t value = i < 6 ? values[i] : (int32_t)(i - 6) - 32768;\n\
             if ({}((_Bool)t, value, 1) != (int32_t)t) return 1;\n\
             if ({}((_Bool)t, value, 0) != (t ? value : 17)) return 2;\n\
             }} }} return 0; }}\n",
                probe.text, probe.keepalive, probe.names[0], probe.names[0],
            ),
        )
        .unwrap();
        for (compiler_index, compiler, sanitize) in [
            (0, PathBuf::from("gcc-14"), false),
            (1, zig.clone(), false),
            (2, PathBuf::from("gcc-14"), true),
        ] {
            for opt in ["-O0", "-O2"] {
                let round = root.join(format!("{case}-{compiler_index}-{opt}"));
                fs::create_dir_all(&round).unwrap();
                let object = round.join("probe.o");
                let binary = round.join("probe");
                let mut compile = Command::new(&compiler);
                compile.current_dir(&round).args([
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
                    "-fstack-usage",
                    opt,
                ]);
                if compiler_index == 1 {
                    compile
                        .args(["-Xclang", "-stack-usage-file", "-Xclang"])
                        .arg(round.join("probe.su"));
                }
                if sanitize {
                    compile.args(["-fsanitize=undefined", "-fno-sanitize-recover=all"]);
                }
                let output = compile
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
                super::super::call_native_reports::check(
                    &round,
                    &object,
                    &probe,
                    &[vec![1, 2, 3], vec![], vec![], vec![]],
                );
                let mut link = Command::new(&compiler);
                if sanitize {
                    link.args(["-fsanitize=undefined", "-fno-sanitize-recover=all"]);
                }
                let output = link.arg(&object).arg("-o").arg(&binary).output().unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let output = Command::new(&binary)
                    .env("UBSAN_OPTIONS", "halt_on_error=1")
                    .output()
                    .unwrap();
                assert!(
                    output.stderr.is_empty(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert_eq!(
                    output.status.code(),
                    Some(match mode {
                        Mutation::None => 0,
                        Mutation::SwappedTag => 1,
                        Mutation::LostPayload => 2,
                        _ => unreachable!(),
                    })
                );
            }
        }
    }
}
