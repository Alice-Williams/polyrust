//! Dependency-free C17 generation from checked portable IR v0.

#![forbid(unsafe_code)]

mod generator;

use std::collections::BTreeMap;

use generator::Generator;
use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, Document as CodeDocument, FileGroup, FileGroupId, FileRole, ImportGroup,
    ImportSet, InjectedHelper, IrVersionRange, LanguageFile, LanguagePackage, LanguagePlugin,
    LanguageRenderer, LanguageSourceFile, OptionsSchema, OutputManifest, RawText, TargetId,
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
    path: &'static str,
    system: bool,
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
                    .map(|import| {
                        if import.system {
                            format!("#include <{}>", import.path)
                        } else {
                            format!("#include {:?}", import.path)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>();
        Ok(CodeDocument::raw_text(RawText::new(groups.join("\n\n"))))
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
                        FileRole::Documentation,
                        README,
                    )],
                )
                .map_err(c_generation_error)?,
                FileGroup::new(
                    c_group("runtime")?,
                    vec![
                        LanguageFile::source(c_runtime_header_file()),
                        LanguageFile::source(c_runtime_source_file()),
                    ],
                )
                .map_err(c_generation_error)?,
                FileGroup::new(
                    c_group("source")?,
                    vec![
                        LanguageFile::source(c_generated_header_file(&generator)),
                        LanguageFile::source(c_generated_source_file(&generator)?),
                    ],
                )
                .map_err(c_generation_error)?,
                FileGroup::new(
                    c_group("tests")?,
                    vec![
                        LanguageFile::source(c_generated_test_file(&generator)?),
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

fn require_c_system(file: &mut LanguageSourceFile<CImport>, path: &'static str) {
    file.require_import(c_system_group(), CImport { path, system: true });
}

fn require_c_local(file: &mut LanguageSourceFile<CImport>, path: &'static str) {
    file.require_import(
        c_local_group(),
        CImport {
            path,
            system: false,
        },
    );
}

fn guarded_preamble(guard: &str, comment: &str) -> CodeDocument {
    CodeDocument::raw_text(RawText::new(format!(
        "#ifndef {guard}\n#define {guard}\n\n{comment}"
    )))
}

fn guarded_epilogue(guard: &str) -> CodeDocument {
    CodeDocument::raw_text(RawText::new(format!("#endif /* {guard} */")))
}

fn c_runtime_header_file() -> LanguageSourceFile<CImport> {
    const GUARD: &str = "POLYRUST_RUNTIME_H";
    let mut file = LanguageSourceFile::new("src/runtime.h", FileRole::Runtime);
    file.set_preamble(guarded_preamble(
        GUARD,
        "/* Dependency-free C17 ownership runtime copied into generated packages. */",
    ));
    for header in ["stdbool.h", "stddef.h", "stdint.h"] {
        require_c_system(&mut file, header);
    }
    file.set_body(CodeDocument::raw_text(
        RawText::new(c_runtime_header_body()),
    ));
    file.set_epilogue(guarded_epilogue(GUARD));
    file
}

fn c_runtime_source_file() -> LanguageSourceFile<CImport> {
    let mut file = LanguageSourceFile::new("src/runtime.c", FileRole::Runtime);
    require_c_system(&mut file, "stdlib.h");
    require_c_system(&mut file, "string.h");
    require_c_local(&mut file, "runtime.h");
    file.set_body(CodeDocument::raw_text(
        RawText::new(c_runtime_source_body()),
    ));
    file
}

fn c_generated_header_file(generator: &Generator<'_>) -> LanguageSourceFile<CImport> {
    let guard = generator.header_guard();
    let mut file = LanguageSourceFile::new("src/generated.h", FileRole::Source);
    file.set_preamble(guarded_preamble(
        &guard,
        "/* Generated by PolyRust from checked IR v0. */",
    ));
    require_c_local(&mut file, "runtime.h");
    file.set_body(CodeDocument::raw_text(RawText::new(generator.header())));
    file.set_epilogue(guarded_epilogue(&guard));
    file
}

fn c_generated_source_file(
    generator: &Generator<'_>,
) -> Result<LanguageSourceFile<CImport>, BackendError> {
    let mut file = LanguageSourceFile::new("src/generated.c", FileRole::Source);
    file.set_preamble(CodeDocument::raw_text(RawText::new(
        "/* Generated by PolyRust from checked IR v0. */",
    )));
    require_c_system(&mut file, "string.h");
    require_c_local(&mut file, "generated.h");
    file.set_body(CodeDocument::raw_text(RawText::new(generator.source()?)));
    Ok(file)
}

fn c_generated_test_file(
    generator: &Generator<'_>,
) -> Result<LanguageSourceFile<CImport>, BackendError> {
    let mut file = LanguageSourceFile::new("tests/generated_test.c", FileRole::Test);
    require_c_system(&mut file, "string.h");
    require_c_local(&mut file, "generated.h");
    file.set_body(CodeDocument::raw_text(RawText::new(generator.tests()?)));
    Ok(file)
}

fn c_conformance_file() -> LanguageSourceFile<CImport> {
    let mut file = LanguageSourceFile::new("tests/conformance_test.c", FileRole::Conformance);
    require_c_system(&mut file, "limits.h");
    require_c_local(&mut file, "runtime.h");
    file.set_body(CodeDocument::raw_text(RawText::new(CONFORMANCE_BODY)));
    file
}

fn c_runtime_header_body() -> &'static str {
    const PREFIX: &str = "#ifndef POLYRUST_RUNTIME_H\n#define POLYRUST_RUNTIME_H\n\n/* Dependency-free C17 ownership runtime copied into generated packages. */\n\n#include <stdbool.h>\n#include <stddef.h>\n#include <stdint.h>\n\n";
    RUNTIME_H
        .strip_prefix(PREFIX)
        .and_then(|body| body.strip_suffix("\n#endif\n"))
        .expect("checked-in C runtime header wrapper matches language IR")
}

fn c_runtime_source_body() -> &'static str {
    const PREFIX: &str = "#include \"runtime.h\"\n\n#include <stdlib.h>\n#include <string.h>\n\n";
    RUNTIME_C
        .strip_prefix(PREFIX)
        .expect("checked-in C runtime source wrapper matches language IR")
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
        assert_eq!(first.canonical_json(), second.canonical_json());
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
        assert_eq!(generated_source.matches("#include <string.h>").count(), 1);
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
