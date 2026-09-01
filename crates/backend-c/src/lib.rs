//! Dependency-free C17 generation from checked portable IR v0.

#![forbid(unsafe_code)]

mod generator;

use std::collections::BTreeMap;

use generator::Generator;
use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CapabilitySupport,
    DeclaredDependency, InjectedHelper, IrVersionRange, OptionsSchema, OutputFile, OutputManifest,
    TargetId,
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
        _options: &BackendOptions,
    ) -> Result<OutputManifest, BackendError> {
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
        OutputManifest::new(
            vec![
                OutputFile::text("README.md", README),
                OutputFile::text("src/runtime.h", RUNTIME_H),
                OutputFile::text("src/runtime.c", RUNTIME_C),
                OutputFile::text("src/generated.h", generator.header()),
                OutputFile::text("src/generated.c", generator.source()?),
                OutputFile::text("tests/generated_test.c", generator.tests()?),
                OutputFile::text("tests/conformance_test.c", CONFORMANCE),
            ],
            Vec::<DeclaredDependency>::new(),
            helpers,
        )
        .map_err(BackendError::UnsupportedCapabilities)
    }
}

const README: &str = "# Generated PolyRust C17 package\n\nDependency-free C17 source with borrowed inputs, allocator-owned outputs, and explicit portable results.\n";

const CONFORMANCE: &str = r#"#include "runtime.h"

#include <limits.h>

int main(void) {
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
