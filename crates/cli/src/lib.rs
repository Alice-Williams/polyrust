#![forbid(unsafe_code)]

//! Stable v0 command-line composition and safe manifest materialization.

mod output;

pub use output::{MaterializeError, materialize};

use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::Path,
    sync::Arc,
};

use portable_check::v0::{Capability, CheckedProgram, check_program};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendRegistry, BackendVersion,
    CapabilitySupport, IrVersionRange, OptionKind, OptionSpec, OptionValue, OptionsSchema,
    OutputFile, OutputManifest, TargetId,
};
use portable_diagnostics::{
    Color, Diagnostic, DiagnosticCode, SourceProvider, SourceRef, explain, render_json,
    render_terminal,
};
use portable_ir::v0::{IrVersion, from_json};

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_INPUT: i32 = 3;
pub const EXIT_BACKEND: i32 = 4;
pub const EXIT_OUTPUT: i32 = 5;

pub const HELP: &str = "polyrust 0.1.0\n\nUSAGE:\n  polyrust check <input.poly.json> [--target <id>] [--json]\n  polyrust emit <input.poly.json> --target <id> --out <directory> [--dry-run] [--json] [--option <name=value>]...\n  polyrust targets [--json]\n  polyrust explain <code> [--json]\n\nEXIT CODES:\n  0 success\n  2 invalid command line\n  3 input parse or semantic check failed\n  4 target, option, preflight, or generation failed\n  5 output transaction failed\n";

const CAPABILITIES: [Capability; 10] = [
    Capability::Bytes,
    Capability::CheckedIntegerArithmetic,
    Capability::ContractDispatch,
    Capability::F64,
    Capability::ImmutableList,
    Capability::Option,
    Capability::Result,
    Capability::UnicodeScalar,
    Capability::WrappingIntegerArithmetic,
    Capability::BoundedIteration,
];

pub fn run(arguments: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    run_with_registry(arguments, stdout, stderr, default_registry())
}

fn run_with_registry(
    arguments: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    registry: BackendRegistry,
) -> i32 {
    let Some(command) = arguments.first().map(String::as_str) else {
        let _ = stderr.write_all(HELP.as_bytes());
        return EXIT_USAGE;
    };
    match command {
        "--help" | "-h" | "help" => write_success(stdout, HELP),
        "check" => command_check(&arguments[1..], stdout, stderr, &registry),
        "emit" => command_emit(&arguments[1..], stdout, stderr, &registry),
        "targets" => command_targets(&arguments[1..], stdout, stderr, &registry),
        "explain" => command_explain(&arguments[1..], stdout, stderr),
        _ => usage_error(stderr, format!("unknown command {command:?}")),
    }
}

fn command_check(
    arguments: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    registry: &BackendRegistry,
) -> i32 {
    let mut positional = Vec::new();
    let mut target = None;
    let mut json = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--json" => json = true,
            "--target" => {
                index += 1;
                let Some(value) = arguments.get(index) else {
                    return usage_error(stderr, "--target requires a value");
                };
                target = Some(value.clone());
            }
            value if value.starts_with('-') => {
                return usage_error(stderr, format!("unknown check option {value:?}"));
            }
            value => positional.push(value.to_owned()),
        }
        index += 1;
    }
    if positional.len() != 1 {
        return usage_error(stderr, "check requires exactly one input path");
    }
    let Some(program) = load_checked(Path::new(&positional[0]), json, stderr) else {
        return EXIT_INPUT;
    };
    if let Some(target) = target {
        let target = match TargetId::parse(target) {
            Ok(target) => target,
            Err(error) => return backend_message(stderr, json, error.to_string()),
        };
        if let Err(error) = registry.preflight_target(&target, &program, &BackendOptions::default())
        {
            return render_backend_error(stderr, json, error);
        }
    }
    if json {
        write_success(stdout, "{\"ok\":true}\n")
    } else {
        write_success(stdout, &format!("checked {}\n", positional[0]))
    }
}

fn command_emit(
    arguments: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    registry: &BackendRegistry,
) -> i32 {
    let mut input = None;
    let mut target = None;
    let mut output = None;
    let mut dry_run = false;
    let mut json = false;
    let mut option_pairs = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--target" | "--out" | "--option" => {
                let flag = arguments[index].clone();
                index += 1;
                let Some(value) = arguments.get(index).cloned() else {
                    return usage_error(stderr, format!("{flag} requires a value"));
                };
                match flag.as_str() {
                    "--target" => target = Some(value),
                    "--out" => output = Some(value),
                    "--option" => option_pairs.push(value),
                    _ => unreachable!(),
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json = true,
            value if value.starts_with('-') => {
                return usage_error(stderr, format!("unknown emit option {value:?}"));
            }
            value if input.is_none() => input = Some(value.to_owned()),
            value => return usage_error(stderr, format!("unexpected argument {value:?}")),
        }
        index += 1;
    }
    let (Some(input), Some(target), Some(output)) = (input, target, output) else {
        return usage_error(stderr, "emit requires input, --target, and --out");
    };
    let target = match TargetId::parse(target) {
        Ok(target) => target,
        Err(error) => return backend_message(stderr, json, error.to_string()),
    };
    let options = match parse_options(option_pairs) {
        Ok(options) => options,
        Err(message) => return usage_error(stderr, message),
    };
    let Some(program) = load_checked(Path::new(&input), json, stderr) else {
        return EXIT_INPUT;
    };
    let manifest = match registry.generate(&target, &program, &options) {
        Ok(manifest) => manifest,
        Err(error) => return render_backend_error(stderr, json, error),
    };
    if dry_run {
        return display_manifest(stdout, &manifest, json, true);
    }
    if let Err(error) = materialize(Path::new(&output), &manifest) {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UnsafeOutputPath,
            format!("output transaction failed: {error}"),
            SourceRef::logical(["cli", "emit", "output"]),
        );
        let _ = write_diagnostics(stderr, json, &[diagnostic]);
        return EXIT_OUTPUT;
    }
    display_manifest(stdout, &manifest, json, false)
}

fn command_targets(
    arguments: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    registry: &BackendRegistry,
) -> i32 {
    let json = match arguments {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => return usage_error(stderr, "targets accepts only --json"),
    };
    let targets: Vec<_> = registry
        .targets()
        .filter_map(|target| registry.get(target))
        .map(|backend| {
            let descriptor = backend.descriptor();
            let support = CAPABILITIES
                .iter()
                .map(|capability| {
                    let level = match backend.support(*capability) {
                        CapabilitySupport::Native => "native".to_owned(),
                        CapabilitySupport::Helper { helper } => format!("helper:{helper}"),
                        CapabilitySupport::Unsupported { reason } => {
                            format!("unsupported:{reason}")
                        }
                    };
                    (format!("{capability:?}"), level)
                })
                .collect::<BTreeMap<_, _>>();
            (descriptor, support)
        })
        .collect();
    let text = if json {
        let values: Vec<_> = targets
            .iter()
            .map(|(descriptor, support)| {
                serde_json::json!({
                    "target": descriptor.target.as_str(),
                    "display_name": descriptor.display_name,
                    "backend_version": format_version(descriptor.backend_version),
                    "ir_minimum": descriptor.supported_ir.minimum.to_string(),
                    "ir_maximum": descriptor.supported_ir.maximum.to_string(),
                    "support": support,
                })
            })
            .collect();
        format!(
            "{}\n",
            serde_json::to_string(&values).expect("target values serialize")
        )
    } else {
        let mut text = String::new();
        for (descriptor, support) in targets {
            text.push_str(&format!(
                "{}\t{}\tbackend {}\tIR {}..={}\n",
                descriptor.target,
                descriptor.display_name,
                format_version(descriptor.backend_version),
                descriptor.supported_ir.minimum,
                descriptor.supported_ir.maximum,
            ));
            for (capability, level) in support {
                text.push_str(&format!("  {capability}\t{level}\n"));
            }
        }
        text
    };
    write_success(stdout, &text)
}

fn command_explain(arguments: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let (code, json) = match arguments {
        [code] => (code.as_str(), false),
        [code, flag] if flag == "--json" => (code.as_str(), true),
        _ => {
            return usage_error(
                stderr,
                "explain requires one diagnostic code and optional --json",
            );
        }
    };
    let Some(code) = DiagnosticCode::ALL
        .into_iter()
        .find(|candidate| candidate.as_str() == code)
    else {
        return usage_error(stderr, format!("unknown diagnostic code {code:?}"));
    };
    let explanation = explain(code);
    let text = if json {
        format!(
            "{}\n",
            serde_json::json!({
                "code": code.as_str(),
                "short": explanation.short,
                "long": explanation.long,
            })
        )
    } else {
        format!("{}: {}\n{}\n", code, explanation.short, explanation.long)
    };
    write_success(stdout, &text)
}

fn load_checked(path: &Path, json: bool, stderr: &mut dyn Write) -> Option<CheckedProgram> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            let diagnostic = Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                format!("cannot read {}: {error}", path.display()),
                SourceRef::logical(["cli", "input"]),
            );
            let _ = write_diagnostics(stderr, json, &[diagnostic]);
            return None;
        }
    };
    let document = match from_json(&bytes) {
        Ok(document) => document,
        Err(error) => {
            let diagnostic = Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                format!("cannot parse {}: {error}", path.display()),
                SourceRef::logical(["cli", "input"]),
            );
            let _ = write_diagnostics(stderr, json, &[diagnostic]);
            return None;
        }
    };
    match check_program(document) {
        Ok(program) => Some(program),
        Err(diagnostics) => {
            let _ = write_diagnostics(stderr, json, &diagnostics);
            None
        }
    }
}

fn parse_options(pairs: Vec<String>) -> Result<BackendOptions, String> {
    let mut values = BTreeMap::new();
    for pair in pairs {
        let Some((name, value)) = pair.split_once('=') else {
            return Err(format!("option {pair:?} must be name=value"));
        };
        if name.is_empty() || values.contains_key(name) {
            return Err(format!("option name {name:?} is empty or duplicated"));
        }
        values.insert(name.to_owned(), OptionValue::Text(value.to_owned()));
    }
    Ok(BackendOptions::new(values))
}

fn display_manifest(
    stdout: &mut dyn Write,
    manifest: &OutputManifest,
    json: bool,
    dry_run: bool,
) -> i32 {
    let text = if json {
        format!(
            "{{\"dry_run\":{dry_run},\"manifest\":{}}}\n",
            manifest.canonical_json()
        )
    } else {
        let mut text = if dry_run {
            "dry run; no files written\n".to_owned()
        } else {
            "output committed\n".to_owned()
        };
        for file in manifest.files() {
            text.push_str(&format!("file\t{}\n", file.path()));
        }
        for dependency in manifest.dependencies() {
            text.push_str(&format!(
                "dependency\t{}\t{}\t{}\n",
                dependency.ecosystem, dependency.name, dependency.requirement
            ));
        }
        for helper in manifest.helpers() {
            text.push_str(&format!("helper\t{}\t{}\n", helper.id, helper.capability));
        }
        text
    };
    write_success(stdout, &text)
}

fn render_backend_error(stderr: &mut dyn Write, json: bool, error: BackendError) -> i32 {
    match error {
        BackendError::UnsupportedCapabilities(diagnostics) => {
            let _ = write_diagnostics(stderr, json, &diagnostics);
        }
        other => {
            let diagnostic = Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                format!("backend request failed: {other:?}"),
                SourceRef::logical(["cli", "backend"]),
            );
            let _ = write_diagnostics(stderr, json, &[diagnostic]);
        }
    }
    EXIT_BACKEND
}

fn backend_message(stderr: &mut dyn Write, json: bool, message: String) -> i32 {
    render_backend_error(stderr, json, BackendError::Generation { message })
}

fn write_diagnostics(stderr: &mut dyn Write, json: bool, diagnostics: &[Diagnostic]) -> bool {
    let text = if json {
        match render_json(diagnostics) {
            Ok(text) => format!("{text}\n"),
            Err(error) => format!("[{{\"message\":{:?}}}]\n", error.to_string()),
        }
    } else {
        diagnostics
            .iter()
            .map(|diagnostic| render_terminal(diagnostic, &NoSources, Color::Never))
            .collect()
    };
    stderr.write_all(text.as_bytes()).is_ok()
}

fn write_success(stdout: &mut dyn Write, text: &str) -> i32 {
    if stdout.write_all(text.as_bytes()).is_ok() {
        EXIT_SUCCESS
    } else {
        EXIT_OUTPUT
    }
}

fn usage_error(stderr: &mut dyn Write, message: impl AsRef<str>) -> i32 {
    let _ = writeln!(stderr, "error: {}\n\n{HELP}", message.as_ref());
    EXIT_USAGE
}

fn format_version(version: BackendVersion) -> String {
    format!("{}.{}.{}", version.major, version.minor, version.patch)
}

struct NoSources;

impl SourceProvider for NoSources {
    fn source(&self, _file: &str) -> Option<String> {
        None
    }
}

fn default_registry() -> BackendRegistry {
    let mut registry = BackendRegistry::default();
    registry
        .register(Arc::new(InspectionBackend {
            target: TargetId::parse("org.polyrust.inspect").expect("static target ID is valid"),
            unicode: CapabilitySupport::Native,
            fail: false,
        }))
        .expect("static target is unique");
    registry
        .register(Arc::new(portable_backend_rust::RustBackend))
        .expect("Rust target is unique");
    registry
}

struct InspectionBackend {
    target: TargetId,
    unicode: CapabilitySupport,
    fail: bool,
}

impl Backend for InspectionBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: self.target.clone(),
            display_name: "Built-in inspection backend".to_owned(),
            backend_version: BackendVersion::new(0, 1, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }

    fn support(&self, capability: Capability) -> CapabilitySupport {
        if capability == Capability::UnicodeScalar {
            self.unicode.clone()
        } else {
            CapabilitySupport::Native
        }
    }

    fn options_schema(&self) -> OptionsSchema {
        BTreeMap::from([(
            "style".to_owned(),
            OptionSpec {
                kind: OptionKind::Choice(BTreeSet::from([
                    "compact".to_owned(),
                    "readable".to_owned(),
                ])),
                required: false,
                description: "Summary layout".to_owned(),
            },
        )])
    }

    fn generate(
        &self,
        program: &CheckedProgram,
        _options: &BackendOptions,
    ) -> Result<OutputManifest, BackendError> {
        if self.fail {
            return Err(BackendError::Generation {
                message: "deliberate inspection failure".to_owned(),
            });
        }
        let summary = serde_json::json!({
            "ir_version": program.document().ir_version.to_string(),
            "module": program.module().name,
            "declarations": program.module().declarations.len(),
            "capabilities": program.capabilities().program().iter().map(|capability| format!("{capability:?}")).collect::<Vec<_>>(),
        });
        OutputManifest::new(
            vec![OutputFile::text(
                "polyrust-inspection.json",
                format!(
                    "{}\n",
                    serde_json::to_string(&summary).expect("summary serializes")
                ),
            )],
            vec![],
            vec![],
        )
        .map_err(BackendError::UnsupportedCapabilities)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use portable_ir::v0::{
        Declaration, DeclarationHeader, Document, MemberHeader, Module, NodeId, NodeMeta,
        RecordDeclaration, SourceRef as IrSourceRef, TypeRef, Visibility, to_canonical_json,
    };

    use portable_codegen::{DeclaredDependency, InjectedHelper};

    use super::*;

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn sandbox(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "polyrust-cli-{name}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    fn write_program(directory: &Path, unicode: bool) -> PathBuf {
        let source = IrSourceRef::logical(["module(example)", "record(Message)"]);
        let declarations = if unicode {
            vec![Declaration::Record(RecordDeclaration {
                header: DeclarationHeader {
                    node: NodeMeta::new(NodeId(1), source.clone()),
                    name: "Message".into(),
                    visibility: Visibility::Public,
                    documentation: vec![],
                },
                fields: vec![portable_ir::v0::FieldDeclaration {
                    header: MemberHeader {
                        node: NodeMeta::new(NodeId(2), source),
                        name: "text".into(),
                        documentation: vec![],
                    },
                    ty: TypeRef::String,
                }],
            })]
        } else {
            vec![]
        };
        let document = Document::new(
            IrVersion::CURRENT,
            Module {
                name: "example".into(),
                declarations,
            },
        );
        let path = directory.join("input.poly.json");
        fs::write(&path, to_canonical_json(&document).unwrap()).unwrap();
        path
    }

    fn write_semantically_invalid_program(directory: &Path) -> PathBuf {
        let document = Document::new(
            IrVersion::CURRENT,
            Module {
                name: "invalid-name".into(),
                declarations: vec![],
            },
        );
        let path = directory.join("invalid-semantics.poly.json");
        fs::write(&path, to_canonical_json(&document).unwrap()).unwrap();
        path
    }

    fn invoke(arguments: Vec<String>, registry: BackendRegistry) -> (i32, String, String) {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = run_with_registry(&arguments, &mut stdout, &mut stderr, registry);
        (
            exit,
            String::from_utf8(stdout).unwrap(),
            String::from_utf8(stderr).unwrap(),
        )
    }

    #[test]
    fn help_targets_and_explain_snapshots_are_stable() {
        assert_eq!(
            invoke(vec!["--help".into()], default_registry()),
            (0, HELP.into(), "".into())
        );
        let targets = invoke(vec!["targets".into()], default_registry());
        assert_eq!(targets.0, 0);
        assert!(targets.1.starts_with(
            "org.polyrust.inspect\tBuilt-in inspection backend\tbackend 0.1.0\tIR 0.1.0..=0.1.0\n"
        ));
        assert!(
            targets
                .1
                .contains("org.polyrust.rust\tRust\tbackend 0.1.0\tIR 0.1.0..=0.1.0\n")
        );
        assert!(
            targets.1.contains(
                "CheckedIntegerArithmetic\thelper:polyrust.runtime.checked-integers.v0\n"
            )
        );
        let explanation = invoke(vec!["explain".into(), "P0404".into()], default_registry());
        assert_eq!(explanation.0, 0);
        assert!(
            explanation
                .1
                .starts_with("P0404: target capability unsupported\n")
        );
    }

    #[test]
    fn check_and_emit_dry_run_make_no_output_changes() {
        let sandbox = sandbox("dry-run");
        let input = write_program(&sandbox, true);
        let output = sandbox.join("out");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("unknown"), "keep").unwrap();
        let before = fs::read(output.join("unknown")).unwrap();
        let check = invoke(
            vec![
                "check".into(),
                input.to_string_lossy().into_owned(),
                "--target".into(),
                "org.polyrust.inspect".into(),
            ],
            default_registry(),
        );
        assert_eq!(check.0, EXIT_SUCCESS);
        let emit = invoke(
            vec![
                "emit".into(),
                input.to_string_lossy().into_owned(),
                "--target".into(),
                "org.polyrust.inspect".into(),
                "--out".into(),
                output.to_string_lossy().into_owned(),
                "--dry-run".into(),
            ],
            default_registry(),
        );
        assert_eq!(emit.0, EXIT_SUCCESS);
        assert_eq!(fs::read(output.join("unknown")).unwrap(), before);
        assert!(!output.join("polyrust-inspection.json").exists());
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn dry_run_manifest_reports_files_dependencies_and_helpers() {
        let manifest = OutputManifest::new(
            vec![OutputFile::text("generated.txt", "ok")],
            vec![DeclaredDependency {
                ecosystem: "example".into(),
                name: "runtime".into(),
                requirement: "1".into(),
            }],
            vec![InjectedHelper {
                id: "unicode".into(),
                capability: "UnicodeScalar".into(),
                files: vec!["generated.txt".into()],
            }],
        )
        .unwrap();
        let mut output = Vec::new();
        assert_eq!(display_manifest(&mut output, &manifest, false, true), 0);
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("file\tgenerated.txt\n"));
        assert!(output.contains("dependency\texample\truntime\t1\n"));
        assert!(output.contains("helper\tunicode\tUnicodeScalar\n"));
    }

    #[test]
    fn emit_commits_and_preserves_unknown_files() {
        let sandbox = sandbox("emit");
        let input = write_program(&sandbox, false);
        let output = sandbox.join("out");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("unknown"), "keep").unwrap();
        let result = invoke(
            vec![
                "emit".into(),
                input.to_string_lossy().into_owned(),
                "--target".into(),
                "org.polyrust.inspect".into(),
                "--out".into(),
                output.to_string_lossy().into_owned(),
            ],
            default_registry(),
        );
        assert_eq!(result.0, EXIT_SUCCESS, "{}", result.2);
        assert_eq!(fs::read_to_string(output.join("unknown")).unwrap(), "keep");
        assert!(output.join("polyrust-inspection.json").exists());
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn parse_check_preflight_generation_and_output_failures_leave_output_unchanged() {
        let sandbox = sandbox("failures");
        let output = sandbox.join("out");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("sentinel"), "unchanged").unwrap();
        let before = fs::read(output.join("sentinel")).unwrap();

        let invalid = sandbox.join("invalid.poly.json");
        fs::write(&invalid, "not json").unwrap();
        let parse = invoke(
            vec!["check".into(), invalid.to_string_lossy().into_owned()],
            default_registry(),
        );
        assert_eq!(parse.0, EXIT_INPUT);

        let invalid_semantics = write_semantically_invalid_program(&sandbox);
        let check = invoke(
            vec![
                "emit".into(),
                invalid_semantics.to_string_lossy().into_owned(),
                "--target".into(),
                "org.polyrust.inspect".into(),
                "--out".into(),
                output.to_string_lossy().into_owned(),
            ],
            default_registry(),
        );
        assert_eq!(check.0, EXIT_INPUT);

        let input = write_program(&sandbox, true);
        let mut limited = BackendRegistry::default();
        limited
            .register(Arc::new(InspectionBackend {
                target: TargetId::parse("org.polyrust.limited").unwrap(),
                unicode: CapabilitySupport::Unsupported {
                    reason: "test".into(),
                },
                fail: false,
            }))
            .unwrap();
        let preflight = invoke(
            vec![
                "emit".into(),
                input.to_string_lossy().into_owned(),
                "--target".into(),
                "org.polyrust.limited".into(),
                "--out".into(),
                output.to_string_lossy().into_owned(),
            ],
            limited,
        );
        assert_eq!(preflight.0, EXIT_BACKEND);

        let mut failing = BackendRegistry::default();
        failing
            .register(Arc::new(InspectionBackend {
                target: TargetId::parse("org.polyrust.failing").unwrap(),
                unicode: CapabilitySupport::Native,
                fail: true,
            }))
            .unwrap();
        let generation = invoke(
            vec![
                "emit".into(),
                input.to_string_lossy().into_owned(),
                "--target".into(),
                "org.polyrust.failing".into(),
                "--out".into(),
                output.to_string_lossy().into_owned(),
            ],
            failing,
        );
        assert_eq!(generation.0, EXIT_BACKEND);

        let output_file = sandbox.join("not-a-directory");
        fs::write(&output_file, "old").unwrap();
        let output_failure = invoke(
            vec![
                "emit".into(),
                input.to_string_lossy().into_owned(),
                "--target".into(),
                "org.polyrust.inspect".into(),
                "--out".into(),
                output_file.to_string_lossy().into_owned(),
            ],
            default_registry(),
        );
        assert_eq!(output_failure.0, EXIT_OUTPUT);
        assert_eq!(fs::read_to_string(output_file).unwrap(), "old");

        assert!(
            OutputManifest::new(vec![OutputFile::text("../invalid", "bad")], vec![], vec![])
                .is_err()
        );
        assert_eq!(fs::read(output.join("sentinel")).unwrap(), before);
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn json_diagnostics_and_discovery_are_valid_and_ansi_free() {
        let sandbox = sandbox("json");
        let invalid = sandbox.join("bad.poly.json");
        fs::write(&invalid, "{").unwrap();
        let result = invoke(
            vec![
                "check".into(),
                invalid.to_string_lossy().into_owned(),
                "--json".into(),
            ],
            default_registry(),
        );
        assert_eq!(result.0, EXIT_INPUT);
        assert!(!result.2.contains('\u{1b}'));
        let value: serde_json::Value = serde_json::from_str(&result.2).unwrap();
        assert!(
            value
                .as_array()
                .is_some_and(|diagnostics| diagnostics.len() == 1)
        );

        let targets = invoke(vec!["targets".into(), "--json".into()], default_registry());
        assert!(serde_json::from_str::<serde_json::Value>(&targets.1).is_ok());
        fs::remove_dir_all(sandbox).unwrap();
    }
}
