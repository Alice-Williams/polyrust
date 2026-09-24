//! Separate producer/consumer translation units, mixed compilers and measured ABI.
use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(super) fn run(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub(super) fn compile(
    compiler: &Path,
    source: &Path,
    object: &Path,
    opt: &str,
    sanitize: bool,
    stack: bool,
) {
    let mut command = Command::new(compiler);
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
        opt,
    ]);
    if stack {
        command.arg("-fstack-usage");
        if compiler.file_name().unwrap() == "zig_native_oracle" {
            command
                .args(["-Xclang", "-stack-usage-file", "-Xclang"])
                .arg(object.with_extension("su"));
        }
    }
    if sanitize {
        command.args(["-fsanitize=undefined", "-fno-sanitize-recover=all"]);
    }
    run(command.arg("-c").arg(source).arg("-o").arg(object));
}

#[cfg(test)]
fn fixture(root: &Path, mode: Mutation) -> super::super::call_native_tests::Probe {
    let linked = linked(mode, PublicApi::ResultSignatures).unwrap();
    let measured = super::super::resources::measure_package(&linked).unwrap();
    let unit = &linked
        .files()
        .iter()
        .find(|f| f.module().key().path.as_str().ends_with(".c"))
        .unwrap()
        .items()[0];
    let names: Vec<_> = ["entry", "construct", "forward", "inspect"]
        .iter()
        .map(|key| {
            unit.spelling
                .functions
                .iter()
                .find(|(f, _)| f.key().name.as_str() == *key)
                .unwrap()
                .1
                .as_str()
                .to_owned()
        })
        .collect();
    let ty = unit
        .spelling
        .types
        .values()
        .next()
        .unwrap()
        .as_str()
        .to_owned();
    let member = |name| {
        unit.spelling
            .values
            .iter()
            .find_map(|(key, value)| match key {
                super::super::bindings::CValueBinding::Member(m)
                    if m.key().name.as_str() == name =>
                {
                    Some(value.as_str().to_owned())
                }
                _ => None,
            })
            .unwrap()
    };
    let tag = member("success");
    let payload = member("payload");
    let frames = measured
        .total
        .function_frames
        .iter()
        .map(|(f, bound)| (unit.spelling.functions[f].as_str().to_owned(), *bound))
        .collect();
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        // Test-only address escape retains the internal helper for native frame/linkage evidence.
        let text = if file.path().ends_with(".c") {
            format!(
                "{text}\nstruct {ty} (*keep_forward)(struct {ty}) = {};\n",
                names[2]
            )
        } else {
            text.clone()
        };
        fs::write(root.join(file.path()), text).unwrap();
    }
    fs::write(
        root.join("header.c"),
        "#include \"polyrust_result.h\"\n#include \"polyrust_result.h\"\n",
    )
    .unwrap();
    fs::write(
        root.join("consumer.c"),
        format!(
            "#include \"polyrust_result.h\"\n\
         int main(void) {{\n\
         const int32_t values[] = {{INT32_MIN, -1, 0, 1, 17, INT32_MAX}};\n\
         for (unsigned int t = 0; t < 2; ++t) {{\n\
         for (unsigned int i = 0; i < 65542; ++i) {{\n\
         int32_t value = i < 6 ? values[i] : (int32_t)(i - 6) - 32768;\n\
         struct {ty} result = {}((_Bool)t, value);\n\
         struct {ty} copy = result;\n\
         if (copy.{tag} != (_Bool)t) return 1;\n\
         if (copy.{payload} != value) return 2;\n\
         if ({}(copy, 1) != (int32_t)t) return 3;\n\
         if ({}(copy, 0) != (t ? value : 17)) return 4;\n\
         if ({}((_Bool)t, value, 1) != (int32_t)t) return 5;\n\
         if ({}((_Bool)t, value, 0) != (t ? value : 17)) return 6;\n\
         }} }} return 0; }}\n",
            names[1], names[3], names[3], names[0], names[0],
        ),
    )
    .unwrap();
    super::super::call_native_tests::Probe {
        text: String::new(),
        names,
        frames,
        bound: measured.total.frame_bound,
        keepalive: String::new(),
    }
}

#[test]
fn public_result_native_separate_and_mixed_compiler_abi_faults_and_bounds() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("public-results");
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    let gcc = PathBuf::from("gcc-14");
    for (case, mode) in [Mutation::None, Mutation::SwappedTag, Mutation::LostPayload]
        .into_iter()
        .enumerate()
    {
        let case_dir = root.join(case.to_string());
        fs::create_dir_all(&case_dir).unwrap();
        let probe = fixture(&case_dir, mode);
        for (index, producer, consumer, sanitize) in [
            (0, &gcc, &gcc, false),
            (1, &zig, &zig, false),
            (2, &gcc, &gcc, true),
            (3, &gcc, &zig, false),
            (4, &zig, &gcc, false),
        ] {
            for opt in ["-O0", "-O2"] {
                let round = case_dir.join(format!("{index}-{opt}"));
                fs::create_dir_all(&round).unwrap();
                let object = round.join("producer.o");
                compile(
                    producer,
                    &case_dir.join("result.c"),
                    &object,
                    opt,
                    sanitize,
                    true,
                );
                super::super::call_native_reports::check_linkage(
                    &round,
                    &object,
                    &probe,
                    &[vec![1, 2, 3], vec![], vec![], vec![]],
                    &[true, true, false, true],
                );
                compile(
                    consumer,
                    &case_dir.join("header.c"),
                    &round.join("header.o"),
                    opt,
                    sanitize,
                    false,
                );
                compile(
                    consumer,
                    &case_dir.join("consumer.c"),
                    &round.join("consumer.o"),
                    opt,
                    sanitize,
                    false,
                );
                let binary = round.join("test");
                let mut link = Command::new(consumer);
                // Zig debug objects contain UBSan checks even without the explicit
                // sanitizer round. A GCC final link must retain their runtime.
                if sanitize || producer == &zig {
                    link.args(["-fsanitize=undefined", "-fno-sanitize-recover=all"]);
                }
                run(link
                    .arg(&object)
                    .arg(round.join("consumer.o"))
                    .arg("-o")
                    .arg(&binary));
                let output = Command::new(binary)
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
