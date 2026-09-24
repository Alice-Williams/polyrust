//! Native trace proof; observer hooks are test scaffolding, never generated APIs.
use super::{
    nominal_fixture,
    public_result_tests::native::{compile, run},
    result_branch_fixture::{self, Mutation},
};
use crate::dialect::*;
use portable_codegen::*;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn files(package: &RenderReadyPackage<CDialect>) -> BTreeMap<String, String> {
    let output = render_certified_package(&CStructuralRenderer, package).unwrap();
    let measured = super::resources::measure_package(package.ast()).unwrap();
    let bytes: usize = output
        .files()
        .iter()
        .map(|file| match file.contents() {
            OutputContents::Text(text) => text.len(),
            _ => panic!("text"),
        })
        .sum();
    assert!(bytes as u64 <= measured.total.source_bound);
    output
        .files()
        .iter()
        .map(|file| {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            (file.path().to_owned(), text.clone())
        })
        .collect()
}

fn observe(text: &mut String, result: &str, name: &str, event: u8) {
    // Exact certified definition names, not a simulation of the AST's calls.
    let prefix = format!("{result} {name}(");
    let definitions: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with(&prefix) && line.ends_with(" {"))
        .map(str::to_owned)
        .collect();
    assert_eq!(
        definitions.len(),
        1,
        "observer must identify one actual definition: {name}"
    );
    let line = &definitions[0];
    let original = format!("{line}\n");
    assert_eq!(text.matches(&original).count(), 1);
    *text = text.replacen(
        &original,
        &format!("{line}\n    poly_test_observe({event});\n"),
        1,
    );
}

#[cfg(test)]
fn driver(entry: &str, trace: bool) -> String {
    let trace = usize::from(trace);
    format!(
        r#"#include "polyrust_branch.h"
#include <stdio.h>
static unsigned int event_count;
static int events[8];
void poly_test_observe(int event);
void poly_test_observe(int event) {{
    if (event_count < 8) events[event_count] = event;
    if (event_count < 9) ++event_count;
}}
static int check(int32_t value, _Bool success) {{
    event_count = 0;
    int32_t output = {entry}(success, value);
    if (output != (success ? value : INT32_C(17))) return 1;
    if ({trace} && (event_count != 2 || events[0] != 1 || events[1] != (success ? 2 : 3))) return 9;
    return 0;
}}
int main(void) {{
    unsigned int observations = 0;
    unsigned int trace_failures = 0;
    const int32_t edges[] = {{INT32_MIN, INT32_MAX, -1, 0, 1, 17}};
    for (int value = -256; value < 256; ++value) {{
        for (int tag = 0; tag < 2; ++tag) {{
            int status = check((int32_t)value, (_Bool)tag);
            if (status == 9) ++trace_failures;
            else if (status) return status;
            ++observations;
        }}
    }}
    for (size_t i = 0; i < sizeof edges / sizeof edges[0]; ++i) {{
        for (int tag = 0; tag < 2; ++tag) {{
            int status = check(edges[i], (_Bool)tag);
            if (status == 9) ++trace_failures;
            else if (status) return status;
            ++observations;
        }}
    }}
    if (observations != 1036) return 2;
    printf("1036 selected-arm observations; %u trace failures\n", trace_failures);
    return trace_failures ? 9 : 0;
}}
"#
    )
}

fn write(root: &Path, files: &BTreeMap<String, String>) {
    fs::create_dir_all(root).unwrap();
    for (path, text) in files {
        fs::write(root.join(path), text).unwrap();
    }
}

fn execute(
    root: &Path,
    compiler: &Path,
    opt: &str,
    expected: i32,
    probe: Option<&super::call_native_tests::Probe>,
) {
    let mut linker = Command::new(compiler);
    linker.args(["-fsanitize=undefined", "-fno-sanitize-recover=all"]);
    for source in ["result", "branch", "driver"] {
        let frames = probe.is_some() && source == "branch";
        let directory = if frames {
            root.join("frames")
        } else {
            root.to_owned()
        };
        fs::create_dir_all(&directory).unwrap();
        let object = directory.join(format!("{source}.o"));
        compile(
            compiler,
            &root.join(format!("{source}.c")),
            &object,
            opt,
            true,
            frames,
        );
        if frames {
            super::call_native_reports::check_linkage(
                &directory,
                &object,
                probe.unwrap(),
                &[vec![1, 2], vec![], vec![]],
                &[true; 3],
            );
        }
        linker.arg(object);
    }
    let binary = root.join("check");
    run(linker.arg("-o").arg(&binary));
    let output = Command::new(binary)
        .env("UBSAN_OPTIONS", "halt_on_error=1")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(expected), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "must not rely on sanitizer faults: {output:?}"
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "1036 selected-arm observations; {} trace failures\n",
            if expected == 0 { 0 } else { 1036 }
        )
    );
}

#[test]
fn selected_arm_native_traces_detect_value_preserving_execution_faults() {
    let producer = nominal_fixture::producer();
    let proof = producer.structs().next().unwrap();
    let constructor = producer
        .functions()
        .find(|f| f.function().key().name.as_str() == "construct")
        .unwrap();
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("result-branch-traces");
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    let gcc = PathBuf::from("gcc-14");
    let mut faults = 0;
    let mut values = 0;
    let mut traces = 0;
    for (case, mode) in [
        Mutation::None,
        Mutation::InactiveArm,
        Mutation::DuplicateScrutinee,
        Mutation::ArmBeforeScrutinee,
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = result_branch_fixture::fixture(&producer, mode);
        let package = nominal_fixture::certify(fixture.input).unwrap();
        let api = CDependencyApi::from_certificate(package).unwrap();
        let symbol = |function| {
            api.functions()
                .find(|proof| proof.function() == function)
                .unwrap()
                .symbol()
                .as_str()
        };
        let entry = symbol(&fixture.entry);
        let measured = super::resources::measure_package(api.package().ast())
            .unwrap()
            .total;
        assert_eq!(measured.function_frames.len(), 3);
        let child_bound = constructor
            .stack_bound_bytes()
            .max(measured.function_frames[&fixture.success])
            .max(measured.function_frames[&fixture.error]);
        assert_eq!(
            measured.frame_bound,
            measured.function_frames[&fixture.entry] + child_bound
        );
        let probe = super::call_native_tests::Probe {
            text: String::new(),
            keepalive: String::new(),
            frames: measured
                .function_frames
                .iter()
                .map(|(function, bound)| (symbol(function).to_owned(), *bound))
                .collect(),
            bound: measured.frame_bound,
            names: [&fixture.entry, &fixture.success, &fixture.error]
                .into_iter()
                .map(|function| symbol(function).to_owned())
                .collect(),
        };
        let mut pristine = files(producer.package());
        pristine.extend(files(api.package()));
        let mut instrumented = pristine.clone();
        observe(
            instrumented.get_mut("result.c").unwrap(),
            &format!("struct {}", proof.symbol().as_str()),
            constructor.symbol().as_str(),
            1,
        );
        observe(
            instrumented.get_mut("branch.c").unwrap(),
            "int32_t",
            symbol(&fixture.success),
            2,
        );
        observe(
            instrumented.get_mut("branch.c").unwrap(),
            "int32_t",
            symbol(&fixture.error),
            3,
        );
        for file in ["result.c", "branch.c"] {
            instrumented
                .get_mut(file)
                .unwrap()
                .insert_str(0, "void poly_test_observe(int);\n");
        }
        for (compiler_index, compiler) in [&gcc, &zig].into_iter().enumerate() {
            for opt in ["-O0", "-O2"] {
                let round = root.join(format!("{case}-{compiler_index}-{opt}"));
                let plain = round.join("plain");
                write(&plain, &pristine);
                fs::write(plain.join("driver.c"), driver(entry, false)).unwrap();
                execute(&plain, compiler, opt, 0, Some(&probe));
                values += 1;
                let observed = round.join("observed");
                write(&observed, &instrumented);
                fs::write(observed.join("driver.c"), driver(entry, true)).unwrap();
                let correct = matches!(mode, Mutation::None);
                execute(&observed, compiler, opt, if correct { 0 } else { 9 }, None);
                if correct {
                    traces += 1;
                } else {
                    faults += 1;
                }
            }
        }
    }
    assert_eq!((values, traces, faults), (16, 4, 12));
}
