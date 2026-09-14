//! Native evidence for the exact bytes returned by the certified call renderer.
use super::{CDialect, call_fixture::scalar_fixture, project_c_package, resources};
use crate::ast::{CObjectType, CObjectTypeKind, CReturnType, CScalarType};
use portable_codegen::{
    TargetLinker, certify_resolved_package, render_certified_package, verify_unresolved_package,
};
use std::{collections::BTreeMap, fs, path::PathBuf, process::Command};

pub(super) struct Probe {
    pub text: String,
    pub frames: BTreeMap<String, u64>,
    pub bound: u64,
    pub names: Vec<String>,
    pub keepalive: String,
}

fn prepare(graph: &[Vec<usize>], locals: usize) -> Result<Probe, u64> {
    prepare_scalar(graph, locals, &vec![1; graph.len()], CScalarType::I32)
}

fn prepare_scalar(
    graph: &[Vec<usize>],
    locals: usize,
    arities: &[usize],
    kind: CScalarType,
) -> Result<Probe, u64> {
    let (registry, source) = scalar_fixture(graph, locals, true, arities, kind);
    let package = project_c_package(registry, vec![source]).unwrap();
    let checked = verify_unresolved_package(&CDialect, package).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let unit = &linked.files()[0].items()[0];
    let measured = resources::measure(unit).unwrap();
    let frames = measured
        .function_frames
        .iter()
        .map(|(function, bound)| {
            (
                unit.spelling.functions[function].as_str().to_owned(),
                *bound,
            )
        })
        .collect();
    // Test-only observable addresses retain optimized helper definitions.
    // Derive exact prototypes from typed signatures; never cast a function pointer.
    let keepalive = measured
        .function_frames
        .keys()
        .enumerate()
        .map(|(index, function)| {
            let CReturnType::Value(result) = function.signature().return_type() else {
                panic!("scalar return")
            };
            let parameters = function
                .signature()
                .parameters()
                .iter()
                .map(|parameter| spelling(parameter.declared_type()))
                .collect::<Vec<_>>();
            let parameters = if parameters.is_empty() {
                "void".into()
            } else {
                parameters.join(", ")
            };
            format!(
                "{} (*poly_probe_keep{index})({parameters}) = {};",
                spelling(result.declared_type()),
                unit.spelling.functions[function].as_str()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let names: Vec<String> = (0..graph.len())
        .map(|index| {
            let function = measured
                .function_frames
                .keys()
                .find(|function| function.key().name.as_str() == format!("call{index}"))
                .unwrap();
            unit.spelling.functions[function].as_str().to_owned()
        })
        .collect();
    let certified = match certify_resolved_package(&CDialect, linked) {
        Ok(value) => value,
        Err(errors) => {
            assert!(errors.iter().all(
                |error| error.code == portable_diagnostics::DiagnosticCode::TargetResourceLimit
            ));
            assert!(measured.frame_bound > 1024 * 1024);
            return Err(measured.frame_bound);
        }
    };
    let output = render_certified_package(&super::CStructuralRenderer, &certified).unwrap();
    let portable_codegen::OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("expected C source");
    };
    assert!(text.len() as u64 <= measured.source_bound);
    Ok(Probe {
        text: text.clone(),
        frames,
        bound: measured.frame_bound,
        names,
        keepalive,
    })
}

fn spelling(ty: &CObjectType) -> &'static str {
    match ty.kind() {
        CObjectTypeKind::Scalar(CScalarType::I32) => "int32_t",
        CObjectTypeKind::Scalar(CScalarType::Int) => "int",
        CObjectTypeKind::Scalar(CScalarType::Bool) => "_Bool",
        _ => panic!("scalar native oracle signature"),
    }
}

fn chain(count: usize) -> Vec<Vec<usize>> {
    (0..count)
        .map(|index| {
            if index + 1 < count {
                vec![index + 1]
            } else {
                vec![]
            }
        })
        .collect()
}

#[test]
fn generated_call_paths_fit_native_frames_and_stack() {
    let mut low = 2;
    let mut high = 128;
    assert!(prepare(&chain(low), 0).is_ok());
    assert!(prepare(&chain(high), 0).is_err());
    while high - low > 1 {
        let middle = (low + high) / 2;
        if prepare(&chain(middle), 0).is_ok() {
            low = middle;
        } else {
            high = middle;
        }
    }
    let maximum = chain(low);
    let boundary = prepare(&maximum, 0).unwrap_or_else(|_| panic!("admitted boundary"));
    let rejected = prepare(&chain(high), 0).err().unwrap();
    eprintln!(
        "C native call-path boundary: {low} functions, {} bytes; {high} functions reject at {rejected} bytes",
        boundary.bound
    );
    let mut cases = vec![
        (vec![vec![1], vec![]], 0, vec![1; 2], CScalarType::I32),
        (
            vec![vec![1, 2], vec![3], vec![3], vec![]],
            0,
            vec![1; 4],
            CScalarType::I32,
        ),
        (chain(8), 8, vec![1; 8], CScalarType::I32),
        (vec![vec![1, 1, 1], vec![]], 8, vec![1; 2], CScalarType::I32),
        (maximum, 0, vec![1; low], CScalarType::I32),
    ];
    for kind in [CScalarType::I32, CScalarType::Int, CScalarType::Bool] {
        for arity in [0, 3, 127] {
            cases.push((chain(2), 0, vec![1, arity], kind));
        }
    }
    let directory =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("c-native-call-paths");
    fs::create_dir_all(&directory).unwrap();
    let version = Command::new("gcc-14")
        .arg("-dumpfullversion")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(String::from_utf8(version.stdout).unwrap().trim(), "14.2.0");
    let zig = PathBuf::from(std::env::var_os("TEST_SRCDIR").unwrap())
        .join(std::env::var_os("TEST_WORKSPACE").unwrap())
        .join("tools/c/zig_native_oracle");
    for (case, (graph, locals, arities, kind)) in cases.iter().enumerate() {
        let probe = prepare_scalar(graph, *locals, arities, *kind)
            .unwrap_or_else(|_| panic!("native fixture admitted"));
        let input = directory.join(format!("case{case}.c"));
        let expected = if arities.contains(&0) {
            "0"
        } else if *kind == CScalarType::Bool {
            "(value != 0)"
        } else {
            "value"
        };
        fs::write(
            &input,
            format!(
                "{}\n{}\nint main(int argc, char **argv) {{\n\
             (void)argv;\n\
             const int32_t inputs[] = {{ INT32_MIN, -1, 0, 1, 42, INT32_MAX, (int32_t)argc }};\n\
             for (unsigned int i = 0; i < sizeof(inputs) / sizeof(inputs[0]); ++i) {{\n\
             int32_t value = inputs[i];\n\
             if ({}(value) != {expected}) {{ return 1; }}\n\
             }}\nreturn 0;\n}}\n",
                probe.text, probe.keepalive, probe.names[0]
            ),
        )
        .unwrap();
        for (compiler_index, compiler, sanitizer) in [
            (0, PathBuf::from("gcc-14"), None),
            (1, zig.clone(), None),
            (2, PathBuf::from("gcc-14"), Some("address")),
            (3, PathBuf::from("gcc-14"), Some("undefined")),
        ] {
            for optimization in ["-O0", "-O2"] {
                let round = directory.join(format!("case{case}_{compiler_index}{optimization}"));
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
                    optimization,
                ]);
                if compiler_index == 1 {
                    compile
                        .args(["-Xclang", "-stack-usage-file", "-Xclang"])
                        .arg(round.join("probe.su"));
                }
                if let Some(kind) = sanitizer {
                    compile.args([
                        format!("-fsanitize={kind}"),
                        "-fno-sanitize-recover=all".into(),
                        "-fno-pie".into(),
                        "-no-pie".into(),
                    ]);
                }
                let result = compile
                    .arg("-c")
                    .arg(&input)
                    .arg("-o")
                    .arg(&object)
                    .output()
                    .unwrap();
                assert!(
                    result.status.success(),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
                super::call_native_reports::check(&round, &object, &probe, graph);
                let mut link = Command::new(&compiler);
                if let Some(kind) = sanitizer {
                    link.args([
                        format!("-fsanitize={kind}"),
                        "-fno-pie".into(),
                        "-no-pie".into(),
                    ]);
                }
                let result = link.arg(&object).arg("-o").arg(&binary).output().unwrap();
                assert!(
                    result.status.success(),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
                assert!(super::native_stack::run(&binary).success());
            }
        }
    }
}
