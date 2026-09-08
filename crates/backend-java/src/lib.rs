//! Java 21 generation through verified CoreIR, a typed Java AST, the shared
//! symbol linker, opaque syntax certification, and total structural rendering.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::wildcard_imports))]

pub mod ast;
pub mod capabilities;
pub mod dialect;
mod lower;
mod preflight;
mod render;
mod runtime;

use std::collections::BTreeMap;

use portable_build::{Requirements, SupportsAll, TypedProgram};
use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CanonicalCoreAdapter,
    CapabilitySupport, CertifiedStructuralRendererAdapter, IrVersionRange, OptionsSchema,
    OutputManifest, TargetId, TargetLinker, TypedCompiler, TypedCompilerAdapter,
    TypedLanguagePlugin,
};
use portable_core_ir::CoreProgram;
use portable_ir::v0::IrVersion;

use crate::{
    capabilities::{JavaCapabilitySet, java_capabilities},
    dialect::{JavaDialect, JavaHelperCapability, JavaRuntimeHelper},
    lower::JavaLowerer,
    preflight::JavaCapabilityRegistry,
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
        TypedCompilerAdapter::new(CanonicalCoreAdapter, JavaPlugin::new())
    }

    /// Generates Java from a valid-by-construction typed portable program.
    ///
    /// The `SupportsAll<R>` bound proves at the call site that Java implements
    /// every feature inferred from this particular program. A failure below
    /// this boundary is a PolyRust implementation defect, not a user diagnostic.
    ///
    /// An ordinary dynamically checked program cannot call this API:
    ///
    /// ```compile_fail
    /// use portable_backend_java::JavaBackend;
    /// use portable_check::v0::CheckedProgram;
    /// fn rejected(program: &CheckedProgram) {
    ///     let _ = JavaBackend.generate_typed(program);
    /// }
    /// ```
    ///
    /// A generic requirement tree without a Java support proof also fails:
    ///
    /// ```compile_fail
    /// use portable_backend_java::JavaBackend;
    /// use portable_build::{Requirements, TypedProgram};
    /// fn rejected<R: Requirements>(program: &TypedProgram<R>) {
    ///     let _ = JavaBackend.generate_typed(program);
    /// }
    /// ```
    pub fn generate_typed<R>(&self, program: &TypedProgram<R>) -> OutputManifest
    where
        R: Requirements,
        JavaPlugin: SupportsAll<R>,
    {
        self.generate(program.checked_program(), &BackendOptions::default())
            .unwrap_or_else(|error| panic!("TypedProgram Java invariant failure: {error:#?}"))
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

#[derive(Clone, Copy, Debug)]
pub struct JavaPlugin {
    features: JavaCapabilitySet,
}

impl JavaPlugin {
    fn new() -> Self {
        Self {
            features: java_capabilities(),
        }
    }
}

impl Default for JavaPlugin {
    fn default() -> Self {
        Self::new()
    }
}

portable_build::delegate_catalogue_support!(
    JavaPlugin,
    features,
    JavaDialect,
    capabilities::JavaCapabilitySlots
);

impl TypedLanguagePlugin<CoreProgram> for JavaPlugin {
    type Dialect = JavaDialect;
    type CapabilityRegistry = JavaCapabilityRegistry;
    type Lowerer = JavaLowerer;
    type Resolver = TargetLinker<JavaDialect>;
    type Renderer = CertifiedStructuralRendererAdapter<JavaDialect, JavaRenderer>;

    fn descriptor(&self) -> BackendDescriptor {
        JavaBackend::descriptor_value()
    }
    fn options_schema(&self) -> OptionsSchema {
        BTreeMap::new()
    }
    fn dialect(&self) -> Self::Dialect {
        JavaDialect
    }
    fn capability_registry(&self) -> Self::CapabilityRegistry {
        JavaCapabilityRegistry::new(self.features)
    }
    fn lowerer(&self) -> Self::Lowerer {
        JavaLowerer::new(self.features)
    }
    fn resolver(&self) -> Self::Resolver {
        TargetLinker::new(JavaDialect)
    }
    fn renderer(&self) -> Self::Renderer {
        CertifiedStructuralRendererAdapter::new(JavaRenderer)
    }
}

#[cfg(test)]
mod tests;
