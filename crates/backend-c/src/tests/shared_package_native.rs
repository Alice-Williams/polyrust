//! Compile the exact certified implementation separately from header consumers.
use super::{
    CDialect, CStructuralRenderer, call_native_tests::Probe, package_fixture,
    package_projection_tests::linked, resources,
};
use portable_codegen::{OutputContents, certify_resolved_package, render_certified_package};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn successful(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
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

#[test]
fn certified_header_pairs_compile_link_and_execute_with_private_symbols() {
    for (case, fixture) in [
        ("scalar", package_fixture::fixture()),
        (
            "private-record",
            package_fixture::with_layout(package_fixture::RecordLayout::Implementation),
        ),
        (
            "long-independent-paths",
            package_fixture::with_paths(&format!("polyrust_{}.h", "a".repeat(104)), "unrelated.c"),
        ),
    ] {
        prove(case, fixture);
    }
}

#[cfg(test)]
fn prove(case: &str, fixture: package_fixture::Fixture) {
    let header_path = fixture.public.file().key().path.as_str().to_owned();
    let implementation_path = fixture.helper.file().key().path.as_str().to_owned();
    let linked = linked(&fixture);
    let measured = resources::measure_package(&linked).unwrap();
    let source = linked
        .files()
        .iter()
        .find(|file| file.module() == fixture.helper.file())
        .unwrap();
    let names = &source.items()[0].spelling.functions;
    let public = names[&fixture.public].as_str().to_owned();
    let helper = names[&fixture.helper].as_str().to_owned();
    let record_name = fixture
        .record
        .as_ref()
        .map(|record| source.items()[0].spelling.types[record].as_str().to_owned());
    let probe = Probe {
        text: String::new(),
        keepalive: String::new(),
        frames: measured
            .total
            .function_frames
            .iter()
            .map(|(function, bound)| (names[function].as_str().to_owned(), *bound))
            .collect(),
        bound: measured.total.frame_bound,
        names: vec![public.clone(), helper.clone()],
    };
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
    let directory = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap())
        .join("c-public-pair")
        .join(case);
    fs::create_dir_all(&directory).unwrap();
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("C text")
        };
        let path = directory.join(file.path());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    // The exact generated implementation is compiled and linked unchanged.
    // A separate test-only probe keeps function addresses observable so that
    // optimized-away trivial calls cannot make frame evidence disappear.
    let retained = directory
        .join(&implementation_path)
        .with_file_name("retained.c");
    fs::write(
        &retained,
        format!(
            "{}\nint32_t (*poly_probe_api)(int32_t) = {public};\n\
        int32_t (*poly_probe_helper)(int32_t) = {helper};\n",
            fs::read_to_string(directory.join(&implementation_path)).unwrap()
        ),
    )
    .unwrap();
    let consumer_directory = directory.join("external-consumer");
    fs::create_dir_all(&consumer_directory).unwrap();
    let consumer = consumer_directory.join("consumer.c");
    fs::write(
        &consumer,
        format!(
            "#include \"{header_path}\"\n#include \"{header_path}\"\n\
        int main(void) {{\n\
        const int32_t values[] = {{ INT32_MIN, -1, 0, 1, 42, INT32_MAX }};\n\
        for (unsigned int i = 0; i < sizeof(values)/sizeof(values[0]); ++i) {{\n\
        if ({public}(values[i]) != values[i]) {{ return 1; }}\n\
        }}\nreturn 0;\n}}\n"
        ),
    )
    .unwrap();
    let inaccessible = consumer_directory.join("private_consumer.c");
    fs::write(
        &inaccessible,
        format!("#include \"{header_path}\"\nint main(void) {{ return {helper}(0); }}\n"),
    )
    .unwrap();
    let private_record = record_name.as_ref().map(|name| {
        let input = consumer_directory.join("private_record.c");
        fs::write(&input, format!("#include \"{header_path}\"\nint main(void) {{ return (int)sizeof(struct {name}); }}\n")).unwrap();
        (input, name)
    });
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
            let object = implementation(
                &compiler,
                index == 1,
                optimization,
                sanitizer,
                &round,
                &directory.join(&implementation_path),
            );
            exact_exports(&object, &public);
            let frame_directory = round.join("frames");
            fs::create_dir_all(&frame_directory).unwrap();
            let frame_object = implementation(
                &compiler,
                index == 1,
                optimization,
                sanitizer,
                &frame_directory,
                &retained,
            );
            super::call_native_reports::check(
                &frame_directory,
                &frame_object,
                &probe,
                &[vec![1], vec![]],
            );
            let consumer_object = round.join("consumer.o");
            let mut compile = Command::new(&compiler);
            options(&mut compile, optimization, sanitizer);
            successful(
                compile
                    .arg("-c")
                    .arg(&consumer)
                    .arg("-I")
                    .arg(&directory)
                    .arg("-o")
                    .arg(&consumer_object),
            );
            let binary = round.join("consumer");
            let mut link = Command::new(&compiler);
            options(&mut link, optimization, sanitizer);
            successful(
                link.arg(&object)
                    .arg(&consumer_object)
                    .arg("-o")
                    .arg(&binary),
            );
            assert!(super::native_stack::run(&binary).success());
            let retained_binary = round.join("retained_consumer");
            let mut link = Command::new(&compiler);
            options(&mut link, optimization, sanitizer);
            successful(
                link.arg(&frame_object)
                    .arg(&consumer_object)
                    .arg("-o")
                    .arg(&retained_binary),
            );
            assert!(super::native_stack::run(&retained_binary).success());
            private_header_control(&compiler, &inaccessible, &round, &directory, &helper);
            if let Some((input, name)) = &private_record {
                private_header_control(&compiler, input, &round, &directory, name);
            }
        }
    }
}

fn implementation(
    compiler: &Path,
    zig: bool,
    optimization: &str,
    sanitizer: Option<&str>,
    directory: &Path,
    input: &Path,
) -> PathBuf {
    let object = directory.join("implementation.o");
    let mut compile = Command::new(compiler);
    options(&mut compile, optimization, sanitizer);
    compile.current_dir(directory).arg("-fstack-usage");
    if zig {
        compile
            .args(["-Xclang", "-stack-usage-file", "-Xclang"])
            .arg(directory.join("implementation.su"));
    }
    successful(compile.arg("-c").arg(input).arg("-o").arg(&object));
    object
}

fn exact_exports(object: &Path, public: &str) {
    let output = Command::new("nm")
        .args(["--defined-only", "--extern-only"])
        .arg(object)
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    let functions: BTreeSet<_> = output
        .lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            (fields.len() == 3 && fields[1] == "T").then(|| fields[2])
        })
        .collect();
    assert_eq!(
        functions,
        BTreeSet::from([public]),
        "private implementation export leak"
    );
}

fn private_header_control(
    compiler: &Path,
    input: &Path,
    round: &Path,
    include: &Path,
    helper: &str,
) {
    let result = Command::new(compiler)
        .args(["-std=c17", "-Werror", "-c"])
        .arg("-I")
        .arg(include)
        .arg(input)
        .arg("-o")
        .arg(round.join("must_not_compile.o"))
        .output()
        .unwrap();
    assert!(
        !result.status.success(),
        "private helper leaked through header"
    );
    assert!(String::from_utf8_lossy(&result.stderr).contains(helper));
}
