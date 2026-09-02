//! Java 21 generation through verified CoreIR, a typed Java AST, the shared
//! symbol linker, and certified strict Handlebars rendering.

#![forbid(unsafe_code)]

pub mod ast;
pub mod dialect;
mod lower;
mod render;
mod runtime;

use std::collections::BTreeMap;

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CanonicalCoreAdapter,
    CapabilitySupport, CertifiedRendererAdapter, IrVersionRange, OptionsSchema, OutputManifest,
    TargetId, TargetLinker, TypedCompiler, TypedCompilerAdapter, TypedLanguagePlugin,
};
use portable_core_ir::CoreProgram;
use portable_ir::v0::IrVersion;

use crate::{
    dialect::{JavaDialect, JavaHelperCapability, JavaRuntimeHelper},
    lower::JavaLowerer,
    render::JavaRenderer,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct JavaBackend;

impl JavaBackend {
    fn descriptor_value() -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.java").expect("static target ID is valid"),
            display_name: "Java".to_owned(),
            backend_version: BackendVersion::new(0, 2, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }

    fn compiler() -> TypedCompilerAdapter<CanonicalCoreAdapter, JavaPlugin> {
        TypedCompilerAdapter::new(CanonicalCoreAdapter, JavaPlugin)
    }
}

impl Backend for JavaBackend {
    fn descriptor(&self) -> BackendDescriptor {
        Self::descriptor_value()
    }

    fn support(&self, capability: Capability) -> CapabilitySupport {
        match capability {
            Capability::CheckedIntegerArithmetic => helper_support(
                JavaRuntimeHelper::CheckedIntegers,
                JavaHelperCapability::CheckedArithmetic,
            ),
            Capability::UnicodeScalar => helper_support(
                JavaRuntimeHelper::Unicode,
                JavaHelperCapability::UnicodeScalars,
            ),
            Capability::ImmutableList => helper_support(
                JavaRuntimeHelper::ImmutableLists,
                JavaHelperCapability::ImmutableLists,
            ),
            Capability::Bytes => helper_support(
                JavaRuntimeHelper::Bytes,
                JavaHelperCapability::ImmutableBytes,
            ),
            Capability::InterfaceDispatch | Capability::FirstClassInterfaceValues => {
                helper_support(
                    JavaRuntimeHelper::Interfaces,
                    JavaHelperCapability::InterfaceDispatch,
                )
            }
            Capability::F64 => helper_support(
                JavaRuntimeHelper::FloatBits,
                JavaHelperCapability::ExactFloatBits,
            ),
            Capability::Option | Capability::Result => helper_support(
                JavaRuntimeHelper::TaggedValues,
                JavaHelperCapability::TaggedValues,
            ),
            Capability::WrappingIntegerArithmetic | Capability::BoundedIteration => {
                CapabilitySupport::Native
            }
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
        Self::compiler()
            .compile_checked(program, options)
            .map_err(|error| BackendError::Generation {
                message: format!("typed Java generation failed: {error:#?}"),
            })
    }
}

fn helper_support(
    helper: JavaRuntimeHelper,
    capability: JavaHelperCapability,
) -> CapabilitySupport {
    CapabilitySupport::Helper {
        helper: format!("{}:{}", helper.name(), capability.name()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct JavaPlugin;

impl TypedLanguagePlugin<CoreProgram> for JavaPlugin {
    type Dialect = JavaDialect;
    type Lowerer = JavaLowerer;
    type Resolver = TargetLinker<JavaDialect>;
    type Renderer = CertifiedRendererAdapter<JavaDialect, JavaRenderer>;

    fn descriptor(&self) -> BackendDescriptor {
        JavaBackend::descriptor_value()
    }
    fn options_schema(&self) -> OptionsSchema {
        BTreeMap::new()
    }
    fn dialect(&self) -> Self::Dialect {
        JavaDialect
    }
    fn lowerer(&self) -> Self::Lowerer {
        JavaLowerer
    }
    fn resolver(&self) -> Self::Resolver {
        TargetLinker::new(JavaDialect)
    }
    fn renderer(&self) -> Self::Renderer {
        CertifiedRendererAdapter::new(JavaRenderer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portable_codegen::OutputContents;
    use std::collections::BTreeSet;

    fn fixture() -> CheckedProgram {
        portable_check::v0::check_program(
            portable_ir::v0::from_json(include_bytes!(
                "../../build/testdata/registration.poly.json"
            ))
            .expect("fixture parses"),
        )
        .expect("fixture checks")
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        match manifest.file(path).expect("generated file").contents() {
            OutputContents::Text(value) => value,
            OutputContents::Bytes(_) => panic!("Java source must be text"),
        }
    }

    #[test]
    fn generated_manifest_is_typed_deterministic_and_dependency_free() {
        let checked = fixture();
        let first = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let third = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(second.canonical_json(), third.canonical_json());
        assert!(first.dependencies().is_empty());
        assert_eq!(first.files().len(), 6);
    }

    #[test]
    fn generated_source_contains_direct_methods_and_no_legacy_interpreter() {
        let manifest = JavaBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let generated = generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Generated.java",
        );
        let runtime = generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Runtime.java",
        );
        assert!(generated.contains("record Label"));
        assert!(generated.contains("interface Renderable"));
        assert!(generated.contains("Runtime.capture"));
        for forbidden in [
            "serde_json",
            "jsonArray",
            "invokeTest",
            "invokeMethod",
            "readConstant",
            "POLYRUST-BEGIN",
            "POLYRUST-END",
        ] {
            assert!(!generated.contains(forbidden), "found {forbidden}");
            assert!(!runtime.contains(forbidden), "found {forbidden}");
        }
    }

    #[test]
    fn imports_are_derived_from_typed_references() {
        let manifest = JavaBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let generated = generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Generated.java",
        );
        assert!(!generated.contains("import java.math.BigInteger;"));
        assert!(!generated.contains("import java.nio.ByteBuffer;"));
        assert_eq!(generated.matches("import java.util.List;").count(), 0);
        assert_eq!(generated.matches("import java.util.Objects;").count(), 1);
        assert!(generated.contains("Objects.requireNonNull(text)"));
    }

    #[test]
    fn runtime_imports_are_exact_and_physically_deduplicated() {
        let manifest = JavaBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let runtime = generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Runtime.java",
        );
        let imports = runtime
            .lines()
            .filter_map(|line| line.strip_prefix("import ")?.strip_suffix(';'))
            .collect::<Vec<_>>();
        let unique = imports.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(imports.len(), unique.len(), "duplicate import: {imports:?}");
        assert_eq!(
            unique,
            BTreeSet::from([
                "java.math.BigInteger",
                "java.nio.ByteBuffer",
                "java.nio.CharBuffer",
                "java.nio.charset.CharacterCodingException",
                "java.nio.charset.Charset",
                "java.nio.charset.CharsetDecoder",
                "java.nio.charset.CodingErrorAction",
                "java.nio.charset.StandardCharsets",
                "java.util.ArrayList",
                "java.util.List",
                "java.util.Objects",
            ])
        );
    }

    #[test]
    fn canonical_interface_and_composition_corpus_is_flat_and_deterministic() {
        let checked = portable_check::v0::check_program(
            portable_build::interface_composition_fixture().document,
        )
        .expect("canonical interface corpus checks");
        let first = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let third = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(second.canonical_json(), third.canonical_json());

        let generated = generated_text(
            &first,
            "src/main/java/org/polyrust/generated/Generated.java",
        );
        assert!(generated.contains("interface Labelled"));
        assert!(generated.contains("interface Measured"));
        assert!(generated.contains(
            "record Label(String text) implements org.polyrust.generated.Runtime.SemanticValue, Labelled, Measured"
        ));
        assert!(generated.contains("record Service(Labelled renderer)"));
        assert!(generated.contains("Objects.requireNonNull(renderer)"));
        assert!(!generated.contains("interface Labelled extends"));
        assert!(!generated.contains("interface Measured extends"));
    }
}
