//! Dependency-free C17 generation from checked portable IR v0.

#![forbid(unsafe_code)]

mod generator;

use std::collections::{BTreeMap, BTreeSet};

use generator::Generator;
use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, ImportGroup, ImportSet,
    InjectedHelper, IrVersionRange, LanguageFile, LanguageFragment, LanguagePackage,
    LanguagePlugin, LanguageRenderer, LanguageSourceFile, OptionsSchema, OutputManifest, RawText,
    RuntimeHelper, RuntimeHelperGraph, SourceFileRole, TargetId, TextFileRole,
    generate_with_plugin,
};
use portable_ir::v0::IrVersion;

const RUNTIME_H: &str = include_str!("runtime.h");
const RUNTIME_C: &str = include_str!("runtime.c");

/// Native C17 backend with borrowed inputs and allocator-owned outputs.
pub struct CBackend;

impl Backend for CBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.c").expect("static target ID is valid"),
            display_name: "C".to_owned(),
            backend_version: BackendVersion::new(0, 1, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }

    fn support(&self, capability: Capability) -> CapabilitySupport {
        match capability {
            Capability::UnicodeScalar => CapabilitySupport::Helper {
                helper: "polyrust.runtime.c.unicode-scalars.v0".into(),
            },
            Capability::Bytes | Capability::ContractDispatch | Capability::F64 => {
                CapabilitySupport::Native
            }
            Capability::CheckedIntegerArithmetic
            | Capability::ImmutableList
            | Capability::Option
            | Capability::Result
            | Capability::WrappingIntegerArithmetic
            | Capability::BoundedIteration => CapabilitySupport::Unsupported {
                reason: "C17 concrete container ABI is available; portable expression and arithmetic lowering remains in M22B"
                    .into(),
            },
        }
    }

    fn options_schema(&self) -> OptionsSchema {
        BTreeMap::new()
    }

    fn generate(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<OutputManifest, BackendError> {
        generate_with_plugin(self, program, options)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[doc(hidden)]
pub struct CImport {
    kind: CImportKind,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CImportKind {
    System { path: String },
    Local { path: String },
}

impl CImport {
    pub fn system(path: &str) -> Result<Self, String> {
        validate_c_include(path, false)?;
        Ok(Self {
            kind: CImportKind::System {
                path: path.to_owned(),
            },
        })
    }

    pub fn local(path: &str) -> Result<Self, String> {
        validate_c_include(path, true)?;
        Ok(Self {
            kind: CImportKind::Local {
                path: path.to_owned(),
            },
        })
    }
}

fn validate_c_include(path: &str, local: bool) -> Result<(), String> {
    let valid = !path.is_empty()
        && !path.starts_with('/')
        && !path.ends_with('/')
        && !path.contains("//")
        && !path.split('/').any(|segment| matches!(segment, "." | ".."))
        && path.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
        })
        && (!local || path.ends_with(".h"));
    if valid {
        Ok(())
    } else {
        Err(format!("invalid C include path {path:?}"))
    }
}

#[doc(hidden)]
pub struct CRenderer;

impl LanguageRenderer<CImport> for CRenderer {
    fn render_imports(&self, imports: &ImportSet<CImport>) -> Result<CodeDocument, String> {
        let groups = imports
            .groups()
            .map(|(_, imports)| {
                imports
                    .iter()
                    .map(|import| match &import.kind {
                        CImportKind::System { path } => format!("#include <{path}>"),
                        CImportKind::Local { path } => format!("#include {path:?}"),
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>();
        Ok(CodeDocument::raw_text(RawText::new(groups.join("\n\n"))))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CCode {
    pub(crate) text: String,
    pub(crate) imports: BTreeSet<(ImportGroup, CImport)>,
    pub(crate) helper_roots: BTreeSet<String>,
}

impl CCode {
    pub(crate) fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    pub(crate) fn with_system(mut self, path: &str) -> Self {
        self.imports.insert((
            c_system_group(),
            CImport::system(path).expect("static C system include is valid"),
        ));
        self
    }

    pub(crate) fn with_local(mut self, path: &str) -> Self {
        self.imports.insert((
            c_local_group(),
            CImport::local(path).expect("static C local include is valid"),
        ));
        self
    }

    pub(crate) fn with_helper_root(mut self, helper: impl Into<String>) -> Self {
        self.helper_roots.insert(helper.into());
        self
    }

    pub(crate) fn sequence(fragments: impl IntoIterator<Item = Self>) -> Self {
        fragments
            .into_iter()
            .fold(Self::default(), |mut combined, fragment| {
                combined.text.push_str(&fragment.text);
                combined.imports.extend(fragment.imports);
                combined.helper_roots.extend(fragment.helper_roots);
                combined
            })
    }

    pub(crate) fn joined(fragments: impl IntoIterator<Item = Self>, separator: &str) -> Self {
        let mut fragments = fragments.into_iter();
        let Some(first) = fragments.next() else {
            return Self::default();
        };
        fragments.fold(first, |mut combined, fragment| {
            combined.text.push_str(separator);
            combined.text.push_str(&fragment.text);
            combined.imports.extend(fragment.imports);
            combined.helper_roots.extend(fragment.helper_roots);
            combined
        })
    }

    pub(crate) fn map_text(mut self, map: impl FnOnce(String) -> String) -> Self {
        self.text = map(self.text);
        self
    }

    pub(crate) fn with_text_from(mut self, dependencies: impl IntoIterator<Item = Self>) -> Self {
        for dependency in dependencies {
            self.imports.extend(dependency.imports);
            self.helper_roots.extend(dependency.helper_roots);
        }
        self
    }

    fn into_fragment(self) -> LanguageFragment<CImport> {
        let mut fragment = LanguageFragment::new(CodeDocument::raw_text(RawText::new(self.text)));
        for (group, import) in self.imports {
            fragment.require_import(group, import);
        }
        for helper in self.helper_roots {
            fragment = fragment.with_helper_root(helper);
        }
        fragment
    }
}

impl std::fmt::Display for CCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl LanguagePlugin for CBackend {
    type Import = CImport;
    type Renderer = CRenderer;

    fn translate(
        &self,
        program: &CheckedProgram,
        _options: &BackendOptions,
    ) -> Result<LanguagePackage<Self::Import>, BackendError> {
        let generator = Generator::new(program);
        generator.validate()?;
        let header = generator.header();
        let source = generator.source()?;
        let tests = generator.tests()?;
        let mut runtime_roots = BTreeSet::from(["runtime.core".to_owned()]);
        runtime_roots.extend(header.helper_roots.iter().cloned());
        runtime_roots.extend(source.helper_roots.iter().cloned());
        runtime_roots.extend(tests.helper_roots.iter().cloned());
        let helpers = program
            .capabilities()
            .program()
            .iter()
            .filter_map(|capability| match self.support(*capability) {
                CapabilitySupport::Helper { helper } => Some(InjectedHelper {
                    id: helper,
                    capability: format!("{capability:?}"),
                    files: vec!["src/runtime.h".into(), "src/runtime.c".into()],
                }),
                CapabilitySupport::Native | CapabilitySupport::Unsupported { .. } => None,
            })
            .collect();
        LanguagePackage::new(
            vec![
                FileGroup::new(
                    c_group("documentation")?,
                    vec![LanguageFile::text(
                        "README.md",
                        TextFileRole::Documentation,
                        README,
                    )],
                )
                .map_err(c_generation_error)?,
                FileGroup::new(
                    c_group("runtime")?,
                    vec![
                        LanguageFile::source(c_runtime_header_file(&runtime_roots)?),
                        LanguageFile::source(c_runtime_source_file(&runtime_roots)?),
                    ],
                )
                .map_err(c_generation_error)?,
                FileGroup::new(
                    c_group("source")?,
                    vec![
                        LanguageFile::source(c_generated_header_file(&generator, header)),
                        LanguageFile::source(c_generated_source_file(source)),
                    ],
                )
                .map_err(c_generation_error)?,
                FileGroup::new(
                    c_group("tests")?,
                    vec![
                        LanguageFile::source(c_generated_test_file(tests)),
                        LanguageFile::source(c_conformance_file()),
                    ],
                )
                .map_err(c_generation_error)?,
            ],
            Vec::<DeclaredDependency>::new(),
            helpers,
        )
        .map_err(c_generation_error)
    }

    fn renderer(&self) -> Self::Renderer {
        CRenderer
    }
}

fn c_generation_error(error: impl std::fmt::Display) -> BackendError {
    BackendError::Generation {
        message: error.to_string(),
    }
}

fn c_group(name: &str) -> Result<FileGroupId, BackendError> {
    FileGroupId::parse(name).map_err(c_generation_error)
}

fn c_system_group() -> ImportGroup {
    ImportGroup::new(10, "system-headers").expect("static import group is valid")
}

fn c_local_group() -> ImportGroup {
    ImportGroup::new(20, "local-headers").expect("static import group is valid")
}

fn guarded_preamble(guard: &str, comment: &str) -> CCode {
    CCode::new(format!("#ifndef {guard}\n#define {guard}\n\n{comment}"))
}

fn guarded_epilogue(guard: &str) -> CCode {
    CCode::new(format!("#endif /* {guard} */"))
}

fn c_runtime_header_file(
    roots: &BTreeSet<String>,
) -> Result<LanguageSourceFile<CImport>, BackendError> {
    const GUARD: &str = "POLYRUST_RUNTIME_H";
    let mut file = LanguageSourceFile::new("src/runtime.h", SourceFileRole::Runtime);
    file.set_preamble(
        guarded_preamble(
            GUARD,
            "/* Dependency-free C17 ownership runtime copied into generated packages. */",
        )
        .into_fragment(),
    );
    file.set_body(
        c_runtime_helper_graph(RUNTIME_H, RuntimeTemplate::Header)?
            .resolve(roots.iter().cloned())
            .map_err(c_generation_error)?,
    );
    file.set_epilogue(guarded_epilogue(GUARD).into_fragment());
    Ok(file)
}

fn c_runtime_source_file(
    roots: &BTreeSet<String>,
) -> Result<LanguageSourceFile<CImport>, BackendError> {
    let mut file = LanguageSourceFile::new("src/runtime.c", SourceFileRole::Runtime);
    file.set_body(
        c_runtime_helper_graph(RUNTIME_C, RuntimeTemplate::Source)?
            .resolve(roots.iter().cloned())
            .map_err(c_generation_error)?,
    );
    Ok(file)
}

#[derive(Clone, Copy)]
enum RuntimeTemplate {
    Header,
    Source,
}

fn c_runtime_helper_graph(
    template: &str,
    kind: RuntimeTemplate,
) -> Result<RuntimeHelperGraph<CImport>, BackendError> {
    const BEGIN: &str = "/* POLYRUST-BEGIN ";
    const END: &str = "/* POLYRUST-END ";
    let mut helpers = Vec::new();
    let mut active: Option<String> = None;
    let mut source = String::new();
    let mut order = 0_u16;
    for line in template.split_inclusive('\n') {
        let marker = line.trim().trim_end_matches('\r');
        if let Some(id) = marker
            .strip_prefix(BEGIN)
            .and_then(|value| value.strip_suffix(" */"))
        {
            if active.is_some() || !source.trim().is_empty() {
                return Err(c_generation_error(format!(
                    "invalid nested or unowned C runtime helper marker {id:?}"
                )));
            }
            active = Some(id.to_owned());
        } else if let Some(id) = marker
            .strip_prefix(END)
            .and_then(|value| value.strip_suffix(" */"))
        {
            let Some(open) = active.take() else {
                return Err(c_generation_error(format!(
                    "unmatched C runtime helper end marker {id:?}"
                )));
            };
            if open != id || source.trim().is_empty() {
                return Err(c_generation_error(format!(
                    "invalid C runtime helper marker {open:?} closed by {id:?}"
                )));
            }
            helpers.push(RuntimeHelper::new(
                open.clone(),
                order,
                c_runtime_section(kind, &open, std::mem::take(&mut source))?.into_fragment(),
            ));
            order = order
                .checked_add(1)
                .expect("C runtime helper order fits u16");
        } else if active.is_some() {
            source.push_str(line);
        } else if !marker.is_empty() {
            return Err(c_generation_error("C runtime text lacks a helper owner"));
        }
    }
    if let Some(open) = active {
        return Err(c_generation_error(format!(
            "unclosed C runtime helper marker {open:?}"
        )));
    }
    let core_dependencies = match kind {
        RuntimeTemplate::Header => vec!["runtime.core.types"],
        RuntimeTemplate::Source => vec![
            "runtime.core.allocator",
            "runtime.core.views",
            "runtime.core.utf8",
            "runtime.core.ownership",
        ],
    };
    helpers.push(RuntimeHelper::new(
        "runtime.core",
        u16::MAX - 2,
        core_dependencies
            .into_iter()
            .fold(CCode::default(), |code, root| code.with_helper_root(root))
            .into_fragment(),
    ));
    RuntimeHelperGraph::new(helpers).map_err(c_generation_error)
}

fn c_runtime_section(
    kind: RuntimeTemplate,
    id: &str,
    source: String,
) -> Result<CCode, BackendError> {
    let code = CCode::new(source);
    let code = match (kind, id) {
        (RuntimeTemplate::Header, "runtime.core.types") => code
            .with_system("stdbool.h")
            .with_system("stddef.h")
            .with_system("stdint.h"),
        (RuntimeTemplate::Header, "runtime.feature.f64") => code
            .with_system("stdbool.h")
            .with_system("stdint.h")
            .with_helper_root("runtime.core"),
        (RuntimeTemplate::Header, "runtime.feature.string-predicates") => code
            .with_system("stdbool.h")
            .with_helper_root("runtime.core"),
        (RuntimeTemplate::Header, "runtime.feature.string-replace-many") => code
            .with_system("stddef.h")
            .with_helper_root("runtime.core"),
        (RuntimeTemplate::Header, id) if id.starts_with("runtime.feature.") => {
            code.with_helper_root("runtime.core")
        }
        (RuntimeTemplate::Source, "runtime.core.allocator") => {
            code.with_system("stdlib.h").with_local("runtime.h")
        }
        (RuntimeTemplate::Source, "runtime.core.ownership") => code.with_system("string.h"),
        (RuntimeTemplate::Source, "runtime.core.views" | "runtime.core.utf8") => code,
        (RuntimeTemplate::Source, "runtime.feature.f64") => code
            .with_system("math.h")
            .with_system("string.h")
            .with_helper_root("runtime.core"),
        (RuntimeTemplate::Source, "runtime.feature.string-strip-prefix") => code
            .with_helper_root("runtime.core")
            .with_helper_root("runtime.feature.string-predicates"),
        (RuntimeTemplate::Source, id) if id.starts_with("runtime.feature.") => code
            .with_system("string.h")
            .with_helper_root("runtime.core"),
        _ => {
            return Err(c_generation_error(format!(
                "unknown C runtime helper {id:?}"
            )));
        }
    };
    Ok(code)
}

fn c_generated_header_file(generator: &Generator<'_>, body: CCode) -> LanguageSourceFile<CImport> {
    let guard = generator.header_guard();
    let mut file = LanguageSourceFile::new("src/generated.h", SourceFileRole::Source);
    file.set_preamble(
        guarded_preamble(&guard, "/* Generated by PolyRust from checked IR v0. */").into_fragment(),
    );
    file.set_body(body.with_local("runtime.h").into_fragment());
    file.set_epilogue(guarded_epilogue(&guard).into_fragment());
    file
}

fn c_generated_source_file(body: CCode) -> LanguageSourceFile<CImport> {
    let mut file = LanguageSourceFile::new("src/generated.c", SourceFileRole::Source);
    file.set_preamble(
        CCode::new("/* Generated by PolyRust from checked IR v0. */").into_fragment(),
    );
    file.set_body(body.with_local("generated.h").into_fragment());
    file
}

fn c_generated_test_file(body: CCode) -> LanguageSourceFile<CImport> {
    let mut file = LanguageSourceFile::new("tests/generated_test.c", SourceFileRole::Test);
    file.set_body(body.with_local("generated.h").into_fragment());
    file
}

fn c_conformance_file() -> LanguageSourceFile<CImport> {
    let mut file = LanguageSourceFile::new("tests/conformance_test.c", SourceFileRole::Conformance);
    file.set_body(
        CCode::new(CONFORMANCE_BODY)
            .with_system("limits.h")
            .with_local("runtime.h")
            .with_helper_root("runtime.core")
            .into_fragment(),
    );
    file
}

const README: &str = "# Generated PolyRust C17 package\n\nDependency-free C17 source with borrowed inputs, allocator-owned outputs, and explicit portable results.\n";

const CONFORMANCE_BODY: &str = r#"int main(void) {
  static const uint8_t astral[] = {UINT8_C(0xf0), UINT8_C(0x9f), UINT8_C(0xa6), UINT8_C(0x80)};
  static const uint8_t invalid[] = {UINT8_C(0xff)};
  size_t scalars = 0U;
  if (!poly_utf8_valid((poly_string_view){astral, sizeof(astral)}, &scalars) || scalars != 1U) return 1;
  if (poly_utf8_valid((poly_string_view){invalid, sizeof(invalid)}, NULL)) return 2;
  if ((int64_t)INT32_MAX + INT64_C(1) != INT64_C(2147483648)) return 3;
  return 0;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use portable_codegen::OutputContents;
    use portable_ir::v0::TypeRef;

    #[test]
    fn descriptor_and_manifest_are_deterministic() {
        let checked = fixture();
        assert_eq!(CBackend.descriptor().target.as_str(), "org.polyrust.c");
        let first = CBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = CBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let third = CBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(second.canonical_json(), third.canonical_json());
        assert!(first.dependencies().is_empty());
        assert!(first.file("src/generated.c").is_some());
    }

    #[test]
    fn aggregate_abi_is_concrete_and_deterministic() {
        let checked = portable_check::v0::check_program(
            portable_ir::v0::from_json(include_bytes!("../test/abi-shapes.poly.json")).unwrap(),
        )
        .unwrap();
        let first = CBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = CBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        let header = match first.file("src/generated.h").unwrap().contents() {
            OutputContents::Text(text) => text,
            OutputContents::Bytes(_) => panic!("generated C header must be text"),
        };
        assert!(header.contains("struct abi_shapes_list__named_1"));
        assert!(header.contains("struct abi_shapes_option__named_1"));
        assert!(header.contains("struct abi_shapes_result__option__string__bytes"));
        assert!(header.contains("typedef enum abi_shapes_Choice_tag"));
        assert!(!header.contains("void *"));
    }

    #[test]
    fn c_includes_and_abi_types_are_validated_fragments() {
        for header in ["stdbool.h", "sys/types.h", "generated.h"] {
            let result = if header == "generated.h" {
                CImport::local(header)
            } else {
                CImport::system(header)
            };
            assert!(result.is_ok(), "{header}");
        }
        for header in [
            "",
            "../escape.h",
            "/absolute.h",
            "bad\\path.h",
            "x.h>\n#include <y",
        ] {
            assert!(CImport::system(header).is_err(), "{header}");
            assert!(CImport::local(header).is_err(), "{header}");
        }
        assert!(CImport::local("not_a_header.hpp").is_err());

        let program = fixture();
        let generator = Generator::new(&program);
        for (ty, expected_text, expected_headers) in [
            (TypeRef::Unit, "registration_unit", &[][..]),
            (TypeRef::Bool, "bool", &["stdbool.h"][..]),
            (TypeRef::I64, "int64_t", &["stdint.h"][..]),
            (TypeRef::String, "poly_string", &[][..]),
        ] {
            let code = generator.ty(&ty);
            assert_eq!(code.text, expected_text);
            assert_eq!(system_headers(&code), string_set(expected_headers));
        }
        for (ty, expected_headers) in [
            (
                TypeRef::List(Box::new(TypeRef::I64)),
                &["stdbool.h", "stddef.h", "stdint.h"][..],
            ),
            (
                TypeRef::Option(Box::new(TypeRef::String)),
                &["stdbool.h"][..],
            ),
            (
                TypeRef::Result {
                    ok: Box::new(TypeRef::Bytes),
                    error: Box::new(TypeRef::String),
                },
                &["stdbool.h"][..],
            ),
        ] {
            let code = generator.composite_shape_header(&ty);
            assert_eq!(system_headers(&code), string_set(expected_headers));
            assert!(code.helper_roots.contains("runtime.core"));
        }
    }

    #[test]
    fn c_runtime_helper_matrix_is_exact_and_minimal() {
        let core = string_set(&["runtime.core"]);
        let header = render_runtime_header(&core);
        assert_eq!(
            include_headers(&header),
            string_set(&["stdbool.h", "stddef.h", "stdint.h"])
        );
        assert!(!header.contains("poly_f64_trunc"));
        assert!(!header.contains("poly_f64_is_negative_zero"));
        assert!(!header.contains("poly_f64_abs"));
        assert!(!header.contains("poly_string_replace_all"));
        let source = render_runtime_source(&core);
        assert_eq!(
            include_headers(&source),
            string_set(&["stdlib.h", "string.h"])
        );
        assert_eq!(source.matches("#include \"runtime.h\"").count(), 1);
        assert!(!source.contains("POLYRUST-"));
        assert!(!source.contains("poly_f64_trunc"));
        assert!(!source.contains("poly_f64_is_negative_zero"));
        assert!(!source.contains("poly_f64_abs"));

        for (root, present, absent) in [
            (
                "runtime.feature.f64",
                "poly_f64_trunc",
                "poly_string_replace_all",
            ),
            (
                "runtime.feature.string-replace-all",
                "poly_string_replace_all",
                "poly_bytes_replace_all",
            ),
            (
                "runtime.feature.bytes-replace-all",
                "poly_bytes_replace_all",
                "poly_string_replace_many",
            ),
            (
                "runtime.feature.string-replace-many",
                "poly_string_replace_many",
                "poly_string_truncate_utf8_bytes",
            ),
            (
                "runtime.feature.string-truncate-utf8",
                "poly_string_truncate_utf8_bytes",
                "poly_string_trim_start",
            ),
            (
                "runtime.feature.string-trim",
                "poly_string_trim_start",
                "poly_string_replace_all",
            ),
        ] {
            let roots = string_set(&[root]);
            let rendered = render_runtime_source(&roots);
            assert!(rendered.contains(present), "{root} lacks {present}");
            assert!(!rendered.contains(absent), "{root} includes {absent}");
            assert!(!rendered.contains("POLYRUST-"));
            if root == "runtime.feature.f64" {
                assert!(rendered.contains("#include <math.h>"));
                assert!(rendered.contains("poly_f64_is_negative_zero"));
                assert!(rendered.contains("poly_f64_abs"));
            } else {
                assert!(!rendered.contains("#include <math.h>"));
            }
        }

        let manifest = CBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let minimal = generated_text(&manifest, "src/runtime.c");
        assert!(!minimal.contains("poly_string_replace_all"));
        assert!(!minimal.contains("poly_f64_trunc"));
        assert!(!minimal.contains("poly_f64_is_negative_zero"));
        assert!(!minimal.contains("poly_f64_abs"));
        assert!(!minimal.contains("#include <math.h>"));
    }

    #[test]
    fn c_includes_and_guards_are_owned_per_language_file() {
        let manifest = CBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let runtime_header = generated_text(&manifest, "src/runtime.h");
        assert_eq!(runtime_header.matches("#include <stdbool.h>").count(), 1);
        assert!(!runtime_header.contains("#include <stdlib.h>"));
        assert_eq!(
            runtime_header.matches("#ifndef POLYRUST_RUNTIME_H").count(),
            1
        );
        assert_eq!(
            runtime_header
                .matches("#endif /* POLYRUST_RUNTIME_H */")
                .count(),
            1
        );

        let runtime_source = generated_text(&manifest, "src/runtime.c");
        assert_eq!(runtime_source.matches("#include \"runtime.h\"").count(), 1);
        assert_eq!(runtime_source.matches("#include <stdlib.h>").count(), 1);
        let generated_header = generated_text(&manifest, "src/generated.h");
        assert_eq!(
            generated_header.matches("#include \"runtime.h\"").count(),
            1
        );
        assert!(!generated_header.contains("#include <string.h>"));
        let generated_source = generated_text(&manifest, "src/generated.c");
        assert_eq!(
            generated_source.matches("#include \"generated.h\"").count(),
            1
        );
        assert!(!generated_source.contains("#include <string.h>"));
        let conformance = generated_text(&manifest, "tests/conformance_test.c");
        assert_eq!(conformance.matches("#include <limits.h>").count(), 1);
        assert!(!conformance.contains("#include <string.h>"));
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        match manifest.file(path).unwrap().contents() {
            OutputContents::Text(text) => text,
            OutputContents::Bytes(_) => panic!("C source must be text"),
        }
    }

    fn system_headers(code: &CCode) -> BTreeSet<String> {
        code.imports
            .iter()
            .filter_map(|(_, import)| match &import.kind {
                CImportKind::System { path } => Some(path.clone()),
                CImportKind::Local { .. } => None,
            })
            .collect()
    }

    fn string_set(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn include_headers(source: &str) -> BTreeSet<String> {
        source
            .lines()
            .filter_map(|line| line.strip_prefix("#include <")?.strip_suffix('>'))
            .map(str::to_owned)
            .collect()
    }

    fn render_runtime_header(roots: &BTreeSet<String>) -> String {
        render_source_file(c_runtime_header_file(roots).unwrap(), "src/runtime.h")
    }

    fn render_runtime_source(roots: &BTreeSet<String>) -> String {
        render_source_file(c_runtime_source_file(roots).unwrap(), "src/runtime.c")
    }

    fn render_source_file(file: LanguageSourceFile<CImport>, path: &str) -> String {
        let group = FileGroup::new(
            FileGroupId::parse("test").unwrap(),
            vec![LanguageFile::source(file)],
        )
        .unwrap();
        let package =
            LanguagePackage::new(vec![group], Vec::<DeclaredDependency>::new(), Vec::new())
                .unwrap();
        let manifest = portable_codegen::render_language_package(&package, &CRenderer).unwrap();
        generated_text(&manifest, path).to_owned()
    }

    fn fixture() -> CheckedProgram {
        portable_check::v0::check_program(
            portable_ir::v0::from_json(include_bytes!(
                "../../build/testdata/registration.poly.json"
            ))
            .unwrap(),
        )
        .unwrap()
    }
}
