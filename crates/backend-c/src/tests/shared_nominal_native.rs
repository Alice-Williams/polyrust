//! Execute independently certified packages through strict, separate native objects.
use super::{
    nominal_fixture as fixture,
    public_result_tests::native::{compile, run},
};
use crate::dialect::*;
use portable_codegen::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn write(root: &Path, package: &RenderReadyPackage<CDialect>) {
    for file in render_certified_package(&CStructuralRenderer, package)
        .unwrap()
        .files()
    {
        let OutputContents::Text(text) = file.contents() else {
            panic!("text")
        };
        fs::write(root.join(file.path()), text).unwrap();
    }
}

#[cfg(test)]
fn driver(producer: &CDependencyApi, consumer: &CDependencyApi) -> String {
    let proof = producer.structs().next().unwrap();
    let ty = proof.symbol().as_str();
    let tag = proof.member_name(&proof.members()[0]).unwrap().as_str();
    let payload = proof.member_name(&proof.members()[1]).unwrap().as_str();
    let names: Vec<_> = consumer.functions().map(|f| f.symbol().as_str()).collect();
    let (call, copy, construct, get_payload, get_tag) =
        (names[0], names[1], names[2], names[3], names[4]);
    format!(
        r#"#include "polyrust_nominal_122.h"
#include "polyrust_nominal_122.h"
#include <stdio.h>
static int check(int32_t value, _Bool success) {{
    struct {ty} original = {call}(success, value);
    struct {ty} copied = {copy}(original);
    struct {ty} local = {construct}(success, value);
    if (original.{tag} != success || original.{payload} != value) return 1;
    if (copied.{tag} != success || copied.{payload} != value) return 2;
    if (local.{tag} != success || local.{payload} != value) return 3;
    if ({get_payload}(original) != value || {get_tag}(original) != success) return 4;
    return 0;
}}
int main(void) {{
    uint64_t count = 0;
    for (int64_t value = -65536; value <= 65536; ++value) {{
        for (int tag = 0; tag < 2; ++tag) {{
            int status = check((int32_t)value, (_Bool)tag);
            if (status) return status;
            ++count;
        }}
    }}
    const int32_t edges[] = {{INT32_MIN, INT32_MIN + 1, INT32_MAX - 1, INT32_MAX}};
    for (size_t i = 0; i < sizeof edges / sizeof edges[0]; ++i) {{
        for (int tag = 0; tag < 2; ++tag) {{
            int status = check(edges[i], (_Bool)tag);
            if (status) return status;
            ++count;
        }}
    }}
    if (count != UINT64_C(262154)) return 5;
    puts("262154 result observations");
    return 0;
}}
"#
    )
}

fn link(compiler: &Path, root: &Path, producer_object: &str, sanitize: bool) -> PathBuf {
    let executable = root.join("check");
    let mut command = Command::new(compiler);
    if sanitize {
        command.args(["-fsanitize=undefined", "-fno-sanitize-recover=all"]);
    }
    for object in [
        producer_object,
        "nominal_121.o",
        "nominal_122.o",
        "driver.o",
    ] {
        command.arg(root.join(object));
    }
    run(command.arg("-o").arg(&executable));
    executable
}

#[test]
fn nominal_producer_relay_consumer_native_abi_and_faults() {
    let producer = fixture::producer();
    let proof = producer.structs().next().unwrap().clone();
    let constructor = producer
        .functions()
        .find(|f| f.function().key().name.as_str() == "construct")
        .unwrap();
    let relay = CDependencyApi::from_certificate(
        fixture::certify(fixture::fixture(
            121,
            std::slice::from_ref(&proof),
            &[fixture::Operation::Call(constructor.clone())],
        ))
        .unwrap(),
    )
    .unwrap();
    let consumer = CDependencyApi::from_certificate(
        fixture::certify(fixture::fixture(
            122,
            &[relay.structs().next().unwrap().clone()],
            &[
                fixture::Operation::Call(relay.functions().next().unwrap().clone()),
                fixture::Operation::Copy,
                fixture::Operation::Construct,
                fixture::Operation::Payload,
                fixture::Operation::Tag,
            ],
        ))
        .unwrap(),
    )
    .unwrap();
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("nominal-imports");
    fs::create_dir_all(&root).unwrap();
    for api in [&producer, &relay, &consumer] {
        write(&root, api.package());
    }
    fs::write(root.join("driver.c"), driver(&producer, &consumer)).unwrap();
    let gcc = PathBuf::from("gcc-14");
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    let mut rounds = 0;
    for (producer_compiler, consumer_compiler) in
        [(&gcc, &gcc), (&zig, &zig), (&gcc, &zig), (&zig, &gcc)]
    {
        for opt in ["-O0", "-O2"] {
            let sanitize = producer_compiler == consumer_compiler;
            for (source, compiler) in [
                ("result", producer_compiler),
                ("nominal_121", consumer_compiler),
                ("nominal_122", consumer_compiler),
                ("driver", consumer_compiler),
            ] {
                compile(
                    compiler,
                    &root.join(format!("{source}.c")),
                    &root.join(format!("{source}.o")),
                    opt,
                    sanitize,
                    false,
                );
            }
            // Zig debug objects carry UBSan checks even in the mixed ABI rounds.
            let executable = link(
                consumer_compiler,
                &root,
                "result.o",
                sanitize || producer_compiler == &zig,
            );
            let output = Command::new(executable)
                .env("UBSAN_OPTIONS", "halt_on_error=1")
                .output()
                .unwrap();
            assert!(output.status.success(), "{:?}", output);
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                "262154 result observations\n"
            );
            rounds += 1;
            if !sanitize {
                continue;
            }
            for mutation in [
                super::result_fixture::Mutation::SwappedTag,
                super::result_fixture::Mutation::LostPayload,
            ] {
                let (registry, files) = super::result_fixture::public_fixture(
                    mutation,
                    super::result_fixture::PublicApi::ResultSignatures,
                );
                let faulty = fixture::certify(fixture::Fixture {
                    registry,
                    files,
                    functions: vec![],
                })
                .unwrap();
                let output = render_certified_package(&CStructuralRenderer, &faulty).unwrap();
                let source = output
                    .files()
                    .iter()
                    .find(|f| f.path().ends_with(".c"))
                    .unwrap();
                let OutputContents::Text(text) = source.contents() else {
                    panic!("text")
                };
                fs::write(root.join("fault.c"), text).unwrap();
                compile(
                    producer_compiler,
                    &root.join("fault.c"),
                    &root.join("fault.o"),
                    opt,
                    true,
                    false,
                );
                let executable = link(consumer_compiler, &root, "fault.o", true);
                let output = Command::new(executable)
                    .env("UBSAN_OPTIONS", "halt_on_error=1")
                    .output()
                    .unwrap();
                assert_eq!(
                    output.status.code(),
                    Some(1),
                    "fault must compile and fail the actual value oracle: {output:?}"
                );
                assert!(
                    output.stderr.is_empty(),
                    "fault must not rely on a sanitizer failure"
                );
            }
        }
    }
    assert_eq!(rounds, 8);
}

#[test]
fn equal_tag_and_ordinary_names_compile_in_complete_dependency_headers() {
    let first = super::nominal_producer::producer(150, &["shared"], "first_api");
    let second = super::nominal_producer::producer(151, &["second"], "shared");
    let dependencies: Vec<_> = first
        .functions()
        .chain(second.functions())
        .cloned()
        .collect();
    let input = super::dependency_fixture::fixture(
        152,
        &[crate::ast::CScalarType::I32; 2],
        &dependencies,
        &[Some(0), Some(1)],
    );
    let consumer = CDependencyApi::from_certificate(
        fixture::certify(fixture::Fixture {
            registry: input.registry,
            files: input.files,
            functions: input.functions,
        })
        .unwrap(),
    )
    .unwrap();
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("nominal-namespaces");
    fs::create_dir_all(&root).unwrap();
    for api in [&first, &second, &consumer] {
        write(&root, api.package());
    }
    let functions: Vec<_> = consumer.functions().map(|f| f.symbol().as_str()).collect();
    fs::write(root.join("driver.c"), format!(
        "#include \"polyrust_dep_152.h\"\nint main(void) {{ return {}(INT32_MIN) != INT32_MIN || {}(INT32_MAX) != INT32_MAX; }}\n",
        functions[0], functions[1],
    )).unwrap();
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for compiler in [&PathBuf::from("gcc-14"), &zig] {
        for opt in ["-O0", "-O2"] {
            let mut linker = Command::new(compiler);
            linker.args(["-fsanitize=undefined", "-fno-sanitize-recover=all"]);
            for source in ["producer_150", "producer_151", "polyrust_dep_152", "driver"] {
                let object = root.join(format!("{source}.o"));
                compile(
                    compiler,
                    &root.join(format!("{source}.c")),
                    &object,
                    opt,
                    true,
                    false,
                );
                linker.arg(object);
            }
            let binary = root.join("check");
            run(linker.arg("-o").arg(&binary));
            run(Command::new(binary).env("UBSAN_OPTIONS", "halt_on_error=1"));
        }
    }
}
