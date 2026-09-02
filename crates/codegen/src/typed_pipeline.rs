//! ADR-0004 phase ownership and the sealed typed compiler adapter.
//!
//! This module supplies phase machinery only. Canonical CoreIR, target AST
//! infrastructure, capabilities, and resolved rendering arrive in later M34A
//! tasks.

use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

use portable_check::v0::CheckedProgram;
use portable_diagnostics::Diagnostic;

use crate::{
    BackendDescriptor, BackendOptions, DeclaredDependency, InjectedHelper, IrVersionRange,
    OptionsSchema, OutputContents, OutputFile, OutputManifest, TargetId, validate_options,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypedPipelineStage {
    CoreLowering,
    CoreVerification,
    TargetLowering,
    UnresolvedVerification,
    Resolution,
    ResolvedVerification,
    RenderViewConstruction,
    Rendering,
    ManifestAssembly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedGenerationError {
    InvalidOptions(Vec<String>),
    IncompatibleIr {
        actual: portable_ir::v0::IrVersion,
        supported: IrVersionRange,
    },
    Phase {
        stage: TypedPipelineStage,
        diagnostics: Vec<Diagnostic>,
    },
}

impl TypedGenerationError {
    fn phase(stage: TypedPipelineStage, mut diagnostics: Vec<Diagnostic>) -> Self {
        portable_diagnostics::sort_diagnostics(&mut diagnostics);
        Self::Phase { stage, diagnostics }
    }
}

pub trait CoreLowerer: Send + Sync + 'static {
    type Core: Send + Sync + 'static;

    fn lower_core(&self, program: &CheckedProgram) -> Result<Self::Core, Vec<Diagnostic>>;
    fn verify_core(&self, core: &Self::Core) -> Result<(), Vec<Diagnostic>>;
}

/// A verified Core value. Only the compiler adapter can construct it.
///
/// Unchecked input cannot be substituted for this phase:
///
/// ~~~compile_fail
/// use portable_codegen::{BackendOptions, TargetDialect, TargetLowerer};
/// use portable_ir::v0::Document;
///
/// fn bypass<D, L>(lowerer: &L, unchecked: &Document, options: &BackendOptions)
/// where
///     D: TargetDialect,
///     L: TargetLowerer<Document, D>,
/// {
///     let _ = lowerer.lower_target(unchecked, options);
/// }
/// ~~~
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedCore<C> {
    value: C,
}

impl<C> VerifiedCore<C> {
    pub fn value(&self) -> &C {
        &self.value
    }

    fn new(value: C) -> Self {
        Self { value }
    }
}

pub trait TargetDialect: Send + Sync + 'static {
    type Unresolved: Send + Sync + 'static;
    type Resolved: Send + Sync + 'static;

    fn verify_unresolved(&self, ast: &Self::Unresolved) -> Result<(), Vec<Diagnostic>>;
    fn verify_resolved(&self, ast: &Self::Resolved) -> Result<(), Vec<Diagnostic>>;
}

pub trait TargetLowerer<C, D: TargetDialect>: Send + Sync + 'static {
    fn lower_target(
        &self,
        core: &VerifiedCore<C>,
        options: &BackendOptions,
    ) -> Result<D::Unresolved, Vec<Diagnostic>>;
}

/// A target package which passed its unresolved-AST verifier.
///
/// Consumers cannot forge this phase:
///
/// ~~~compile_fail
/// use portable_codegen::{TargetDialect, UnresolvedPackage};
///
/// struct Dialect;
/// impl TargetDialect for Dialect {
///     type Unresolved = ();
///     type Resolved = ();
///     fn verify_unresolved(&self, _: &()) -> Result<(), Vec<portable_diagnostics::Diagnostic>> {
///         Ok(())
///     }
///     fn verify_resolved(&self, _: &()) -> Result<(), Vec<portable_diagnostics::Diagnostic>> {
///         Ok(())
///     }
/// }
/// let _ = UnresolvedPackage::<Dialect> {
///     ast: (),
///     dialect: std::marker::PhantomData,
/// };
/// ~~~
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedPackage<D: TargetDialect> {
    ast: D::Unresolved,
    dialect: PhantomData<fn() -> D>,
}

impl<D: TargetDialect> UnresolvedPackage<D> {
    pub fn ast(&self) -> &D::Unresolved {
        &self.ast
    }

    fn new(ast: D::Unresolved) -> Self {
        Self {
            ast,
            dialect: PhantomData,
        }
    }
}

pub trait TargetResolver<D: TargetDialect>: Send + Sync + 'static {
    fn resolve_target(
        &self,
        package: &UnresolvedPackage<D>,
    ) -> Result<D::Resolved, Vec<Diagnostic>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPackage<D: TargetDialect> {
    ast: D::Resolved,
    dialect: PhantomData<fn() -> D>,
}

impl<D: TargetDialect> ResolvedPackage<D> {
    pub fn ast(&self) -> &D::Resolved {
        &self.ast
    }

    fn new(ast: D::Resolved) -> Self {
        Self {
            ast,
            dialect: PhantomData,
        }
    }
}

/// Render-view construction cannot accept the unresolved package type:
///
/// ~~~compile_fail
/// use portable_codegen::{TargetDialect, TargetRenderer, UnresolvedPackage};
///
/// fn bypass<D, R>(renderer: &R, package: &UnresolvedPackage<D>)
/// where
///     D: TargetDialect,
///     R: TargetRenderer<D>,
/// {
///     let _ = renderer.build_render_view(package);
/// }
/// ~~~
pub trait TargetRenderer<D: TargetDialect>: Send + Sync + 'static {
    type View: Send + Sync + 'static;

    fn build_render_view(
        &self,
        package: &ResolvedPackage<D>,
    ) -> Result<Self::View, Vec<Diagnostic>>;
    fn render_view(
        &self,
        view: &RenderView<D, Self::View>,
    ) -> Result<RenderedPackage, Vec<Diagnostic>>;
}

/// A language-owned render view built from a verified resolved package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderView<D: TargetDialect, V> {
    value: V,
    dialect: PhantomData<fn() -> D>,
}

impl<D: TargetDialect, V> RenderView<D, V> {
    pub fn value(&self) -> &V {
        &self.value
    }

    fn new(value: V) -> Self {
        Self {
            value,
            dialect: PhantomData,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedFile {
    path: String,
    contents: OutputContents,
}

impl RenderedFile {
    pub fn text(path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            contents: OutputContents::Text(contents.into()),
        }
    }

    pub fn bytes(path: impl Into<String>, contents: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            contents: OutputContents::Bytes(contents.into()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderedPackage {
    files: Vec<RenderedFile>,
    dependencies: Vec<DeclaredDependency>,
    helpers: Vec<InjectedHelper>,
}

impl RenderedPackage {
    pub fn new(
        files: Vec<RenderedFile>,
        dependencies: Vec<DeclaredDependency>,
        helpers: Vec<InjectedHelper>,
    ) -> Self {
        Self {
            files,
            dependencies,
            helpers,
        }
    }
}

pub trait TypedLanguagePlugin<C>: Send + Sync + 'static {
    type Dialect: TargetDialect;
    type Lowerer: TargetLowerer<C, Self::Dialect>;
    type Resolver: TargetResolver<Self::Dialect>;
    type Renderer: TargetRenderer<Self::Dialect>;

    fn descriptor(&self) -> BackendDescriptor;
    fn options_schema(&self) -> OptionsSchema;
    fn dialect(&self) -> Self::Dialect;
    fn lowerer(&self) -> Self::Lowerer;
    fn resolver(&self) -> Self::Resolver;
    fn renderer(&self) -> Self::Renderer;
}

pub struct TypedCompilerAdapter<L, P>
where
    L: CoreLowerer,
    P: TypedLanguagePlugin<L::Core>,
{
    core_lowerer: L,
    plugin: P,
}

impl<L, P> TypedCompilerAdapter<L, P>
where
    L: CoreLowerer,
    P: TypedLanguagePlugin<L::Core>,
{
    pub fn new(core_lowerer: L, plugin: P) -> Self {
        Self {
            core_lowerer,
            plugin,
        }
    }

    fn compile(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<OutputManifest, TypedGenerationError> {
        let descriptor = self.plugin.descriptor();
        let actual = program.document().ir_version;
        if !descriptor.supported_ir.contains(actual) {
            return Err(TypedGenerationError::IncompatibleIr {
                actual,
                supported: descriptor.supported_ir,
            });
        }
        let option_errors = validate_options(&self.plugin.options_schema(), options);
        if !option_errors.is_empty() {
            return Err(TypedGenerationError::InvalidOptions(option_errors));
        }

        let core = self
            .core_lowerer
            .lower_core(program)
            .map_err(|diagnostics| {
                TypedGenerationError::phase(TypedPipelineStage::CoreLowering, diagnostics)
            })?;
        self.core_lowerer
            .verify_core(&core)
            .map_err(|diagnostics| {
                TypedGenerationError::phase(TypedPipelineStage::CoreVerification, diagnostics)
            })?;
        let core = VerifiedCore::new(core);

        let dialect = self.plugin.dialect();
        let unresolved =
            self.plugin
                .lowerer()
                .lower_target(&core, options)
                .map_err(|diagnostics| {
                    TypedGenerationError::phase(TypedPipelineStage::TargetLowering, diagnostics)
                })?;
        dialect
            .verify_unresolved(&unresolved)
            .map_err(|diagnostics| {
                TypedGenerationError::phase(TypedPipelineStage::UnresolvedVerification, diagnostics)
            })?;
        let unresolved = UnresolvedPackage::new(unresolved);

        let resolved =
            self.plugin
                .resolver()
                .resolve_target(&unresolved)
                .map_err(|diagnostics| {
                    TypedGenerationError::phase(TypedPipelineStage::Resolution, diagnostics)
                })?;
        dialect.verify_resolved(&resolved).map_err(|diagnostics| {
            TypedGenerationError::phase(TypedPipelineStage::ResolvedVerification, diagnostics)
        })?;
        let resolved = ResolvedPackage::new(resolved);

        let renderer = self.plugin.renderer();
        let render_view = renderer
            .build_render_view(&resolved)
            .map(RenderView::new)
            .map_err(|diagnostics| {
                TypedGenerationError::phase(TypedPipelineStage::RenderViewConstruction, diagnostics)
            })?;
        let rendered = renderer.render_view(&render_view).map_err(|diagnostics| {
            TypedGenerationError::phase(TypedPipelineStage::Rendering, diagnostics)
        })?;
        assemble_manifest(rendered).map_err(|diagnostics| {
            TypedGenerationError::phase(TypedPipelineStage::ManifestAssembly, diagnostics)
        })
    }
}

mod private {
    pub trait Sealed {}
}

/// Object-safe typed compiler entry. It is sealed so plugins must use the
/// generic adapter instead of implementing a bypass.
///
/// ~~~compile_fail
/// use portable_codegen::{
///     BackendDescriptor, BackendOptions, OutputManifest, TypedCompiler,
///     TypedGenerationError,
/// };
/// use portable_check::v0::CheckedProgram;
///
/// struct Bypass;
/// impl TypedCompiler for Bypass {
///     fn descriptor(&self) -> BackendDescriptor { todo!() }
///     fn compile_checked(
///         &self,
///         _: &CheckedProgram,
///         _: &BackendOptions,
///     ) -> Result<OutputManifest, TypedGenerationError> { todo!() }
/// }
/// ~~~
pub trait TypedCompiler: private::Sealed + Send + Sync {
    fn descriptor(&self) -> BackendDescriptor;
    fn compile_checked(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<OutputManifest, TypedGenerationError>;
}

impl<L, P> private::Sealed for TypedCompilerAdapter<L, P>
where
    L: CoreLowerer,
    P: TypedLanguagePlugin<L::Core>,
{
}

impl<L, P> TypedCompiler for TypedCompilerAdapter<L, P>
where
    L: CoreLowerer,
    P: TypedLanguagePlugin<L::Core>,
{
    fn descriptor(&self) -> BackendDescriptor {
        self.plugin.descriptor()
    }

    fn compile_checked(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<OutputManifest, TypedGenerationError> {
        self.compile(program, options)
    }
}

#[derive(Default)]
pub struct TypedBackendRegistry {
    compilers: BTreeMap<TargetId, Arc<dyn TypedCompiler>>,
}

impl TypedBackendRegistry {
    pub fn register(&mut self, compiler: Arc<dyn TypedCompiler>) -> Result<(), TypedRegistryError> {
        let target = compiler.descriptor().target;
        if self.compilers.contains_key(&target) {
            return Err(TypedRegistryError::DuplicateTarget(target));
        }
        self.compilers.insert(target, compiler);
        Ok(())
    }

    pub fn targets(&self) -> impl Iterator<Item = &TargetId> {
        self.compilers.keys()
    }

    pub fn compile(
        &self,
        target: &TargetId,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<OutputManifest, TypedRegistryGenerationError> {
        let compiler = self
            .compilers
            .get(target)
            .ok_or_else(|| TypedRegistryGenerationError::UnknownTarget(target.clone()))?;
        compiler
            .compile_checked(program, options)
            .map_err(TypedRegistryGenerationError::Generation)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedRegistryError {
    DuplicateTarget(TargetId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedRegistryGenerationError {
    UnknownTarget(TargetId),
    Generation(TypedGenerationError),
}

fn assemble_manifest(rendered: RenderedPackage) -> Result<OutputManifest, Vec<Diagnostic>> {
    let files = rendered
        .files
        .into_iter()
        .map(|file| match file.contents {
            OutputContents::Text(contents) => OutputFile::text(file.path, contents),
            OutputContents::Bytes(contents) => OutputFile::bytes(file.path, contents),
        })
        .collect();
    OutputManifest::new(files, rendered.dependencies, rendered.helpers)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use portable_diagnostics::{DiagnosticCode, SourceRef};
    use portable_ir::v0::{Document, IrVersion, Module};

    use super::*;
    use crate::{BackendVersion, OptionKind, OptionSpec, OptionValue};

    fn checked_named(name: &str) -> CheckedProgram {
        portable_check::v0::check_program(Document::new(
            IrVersion::CURRENT,
            Module {
                name: name.into(),
                declarations: vec![],
            },
        ))
        .unwrap()
    }

    fn checked_program() -> CheckedProgram {
        checked_named("typed_pipeline")
    }

    fn diagnostic(phase: &str) -> Diagnostic {
        Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            format!("deliberate {phase} failure"),
            SourceRef::logical(["typed_pipeline", phase]),
        )
    }

    type Trace = Arc<Mutex<Vec<&'static str>>>;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestCore(String);

    #[derive(Clone)]
    struct TestCoreLowerer(Trace);

    impl CoreLowerer for TestCoreLowerer {
        type Core = TestCore;

        fn lower_core(&self, program: &CheckedProgram) -> Result<Self::Core, Vec<Diagnostic>> {
            self.0.lock().unwrap().push("core.lower");
            if program.module().name == "fail_core_lowering" {
                return Err(vec![diagnostic("core.lower")]);
            }
            Ok(TestCore(program.module().name.clone()))
        }

        fn verify_core(&self, core: &Self::Core) -> Result<(), Vec<Diagnostic>> {
            self.0.lock().unwrap().push("core.verify");
            (core.0 != "bad_core")
                .then_some(())
                .ok_or_else(|| vec![diagnostic("core.verify")])
        }
    }

    #[derive(Clone)]
    struct TestDialect(Trace);

    impl TargetDialect for TestDialect {
        type Unresolved = String;
        type Resolved = String;

        fn verify_unresolved(&self, ast: &Self::Unresolved) -> Result<(), Vec<Diagnostic>> {
            self.0.lock().unwrap().push("unresolved.verify");
            (ast != "bad-unresolved")
                .then_some(())
                .ok_or_else(|| vec![diagnostic("unresolved.verify")])
        }

        fn verify_resolved(&self, ast: &Self::Resolved) -> Result<(), Vec<Diagnostic>> {
            self.0.lock().unwrap().push("resolved.verify");
            (ast != "bad-resolved")
                .then_some(())
                .ok_or_else(|| vec![diagnostic("resolved.verify")])
        }
    }

    #[derive(Clone)]
    struct TestLowerer {
        trace: Trace,
        value: String,
    }

    impl TargetLowerer<TestCore, TestDialect> for TestLowerer {
        fn lower_target(
            &self,
            core: &VerifiedCore<TestCore>,
            _options: &BackendOptions,
        ) -> Result<String, Vec<Diagnostic>> {
            self.trace.lock().unwrap().push("target.lower");
            assert_eq!(core.value().0, "typed_pipeline");
            if self.value == "fail-target-lowering" {
                return Err(vec![diagnostic("target.lower")]);
            }
            Ok(self.value.clone())
        }
    }

    #[derive(Clone)]
    struct TestResolver(Trace);

    impl TargetResolver<TestDialect> for TestResolver {
        fn resolve_target(
            &self,
            package: &UnresolvedPackage<TestDialect>,
        ) -> Result<String, Vec<Diagnostic>> {
            self.0.lock().unwrap().push("resolve");
            match package.ast().as_str() {
                "fail-resolution" => Err(vec![diagnostic("resolve")]),
                "bad-resolved" => Ok("bad-resolved".into()),
                value => Ok(format!("resolved:{value}")),
            }
        }
    }

    #[derive(Clone)]
    struct TestRenderer {
        trace: Trace,
        invalid_path: bool,
    }

    impl TargetRenderer<TestDialect> for TestRenderer {
        type View = String;

        fn build_render_view(
            &self,
            package: &ResolvedPackage<TestDialect>,
        ) -> Result<Self::View, Vec<Diagnostic>> {
            self.trace.lock().unwrap().push("view");
            if package.ast() == "resolved:fail-render-view" {
                return Err(vec![diagnostic("view")]);
            }
            Ok(package.ast().clone())
        }

        fn render_view(
            &self,
            view: &RenderView<TestDialect, Self::View>,
        ) -> Result<RenderedPackage, Vec<Diagnostic>> {
            self.trace.lock().unwrap().push("render");
            if view.value() == "resolved:fail-rendering" {
                return Err(vec![diagnostic("render")]);
            }
            let path = if self.invalid_path {
                "../escape"
            } else {
                "src/generated.test"
            };
            Ok(RenderedPackage::new(
                vec![RenderedFile::text(path, format!("{}\n", view.value()))],
                vec![],
                vec![],
            ))
        }
    }

    #[derive(Clone)]
    struct TestPlugin {
        trace: Trace,
        unresolved: String,
        invalid_path: bool,
    }

    impl TypedLanguagePlugin<TestCore> for TestPlugin {
        type Dialect = TestDialect;
        type Lowerer = TestLowerer;
        type Resolver = TestResolver;
        type Renderer = TestRenderer;

        fn descriptor(&self) -> BackendDescriptor {
            BackendDescriptor {
                target: TargetId::parse("org.polyrust.typed-test").unwrap(),
                display_name: "Typed test".into(),
                backend_version: BackendVersion::new(0, 1, 0),
                supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
            }
        }

        fn options_schema(&self) -> OptionsSchema {
            BTreeMap::from([(
                "strict".into(),
                OptionSpec {
                    kind: OptionKind::Boolean,
                    required: false,
                    description: "test option".into(),
                },
            )])
        }

        fn dialect(&self) -> Self::Dialect {
            TestDialect(self.trace.clone())
        }

        fn lowerer(&self) -> Self::Lowerer {
            TestLowerer {
                trace: self.trace.clone(),
                value: self.unresolved.clone(),
            }
        }

        fn resolver(&self) -> Self::Resolver {
            TestResolver(self.trace.clone())
        }

        fn renderer(&self) -> Self::Renderer {
            TestRenderer {
                trace: self.trace.clone(),
                invalid_path: self.invalid_path,
            }
        }
    }

    fn adapter(unresolved: &str, invalid_path: bool) -> (Trace, Arc<dyn TypedCompiler>, TargetId) {
        let trace = Arc::new(Mutex::new(vec![]));
        let adapter = TypedCompilerAdapter::new(
            TestCoreLowerer(trace.clone()),
            TestPlugin {
                trace: trace.clone(),
                unresolved: unresolved.into(),
                invalid_path,
            },
        );
        let target = adapter.descriptor().target;
        (trace, Arc::new(adapter), target)
    }

    #[test]
    fn adapter_runs_every_phase_once_in_order_and_is_deterministic() {
        let (trace, compiler, target) = adapter("program", false);
        let mut registry = TypedBackendRegistry::default();
        registry.register(compiler).unwrap();
        let first = registry
            .compile(&target, &checked_program(), &BackendOptions::default())
            .unwrap();
        assert_eq!(
            trace.lock().unwrap().as_slice(),
            [
                "core.lower",
                "core.verify",
                "target.lower",
                "unresolved.verify",
                "resolve",
                "resolved.verify",
                "view",
                "render",
            ]
        );
        trace.lock().unwrap().clear();
        let second = registry
            .compile(&target, &checked_program(), &BackendOptions::default())
            .unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn failed_verification_stops_later_phases_and_returns_stage() {
        let (trace, compiler, target) = adapter("bad-unresolved", false);
        let mut registry = TypedBackendRegistry::default();
        registry.register(compiler).unwrap();
        let error = registry
            .compile(&target, &checked_program(), &BackendOptions::default())
            .unwrap_err();
        assert!(matches!(
            error,
            TypedRegistryGenerationError::Generation(TypedGenerationError::Phase {
                stage: TypedPipelineStage::UnresolvedVerification,
                ..
            })
        ));
        assert_eq!(
            trace.lock().unwrap().as_slice(),
            [
                "core.lower",
                "core.verify",
                "target.lower",
                "unresolved.verify"
            ]
        );
    }

    #[test]
    fn every_phase_failure_stops_the_remaining_pipeline() {
        struct Case {
            input: &'static str,
            program: &'static str,
            stage: TypedPipelineStage,
            trace: &'static [&'static str],
        }

        let cases = [
            Case {
                input: "program",
                program: "fail_core_lowering",
                stage: TypedPipelineStage::CoreLowering,
                trace: &["core.lower"],
            },
            Case {
                input: "program",
                program: "bad_core",
                stage: TypedPipelineStage::CoreVerification,
                trace: &["core.lower", "core.verify"],
            },
            Case {
                input: "fail-target-lowering",
                program: "typed_pipeline",
                stage: TypedPipelineStage::TargetLowering,
                trace: &["core.lower", "core.verify", "target.lower"],
            },
            Case {
                input: "bad-unresolved",
                program: "typed_pipeline",
                stage: TypedPipelineStage::UnresolvedVerification,
                trace: &[
                    "core.lower",
                    "core.verify",
                    "target.lower",
                    "unresolved.verify",
                ],
            },
            Case {
                input: "fail-resolution",
                program: "typed_pipeline",
                stage: TypedPipelineStage::Resolution,
                trace: &[
                    "core.lower",
                    "core.verify",
                    "target.lower",
                    "unresolved.verify",
                    "resolve",
                ],
            },
            Case {
                input: "bad-resolved",
                program: "typed_pipeline",
                stage: TypedPipelineStage::ResolvedVerification,
                trace: &[
                    "core.lower",
                    "core.verify",
                    "target.lower",
                    "unresolved.verify",
                    "resolve",
                    "resolved.verify",
                ],
            },
            Case {
                input: "fail-render-view",
                program: "typed_pipeline",
                stage: TypedPipelineStage::RenderViewConstruction,
                trace: &[
                    "core.lower",
                    "core.verify",
                    "target.lower",
                    "unresolved.verify",
                    "resolve",
                    "resolved.verify",
                    "view",
                ],
            },
            Case {
                input: "fail-rendering",
                program: "typed_pipeline",
                stage: TypedPipelineStage::Rendering,
                trace: &[
                    "core.lower",
                    "core.verify",
                    "target.lower",
                    "unresolved.verify",
                    "resolve",
                    "resolved.verify",
                    "view",
                    "render",
                ],
            },
        ];

        for case in cases {
            let (trace, compiler, target) = adapter(case.input, false);
            let mut registry = TypedBackendRegistry::default();
            registry.register(compiler).unwrap();
            let error = registry
                .compile(
                    &target,
                    &checked_named(case.program),
                    &BackendOptions::default(),
                )
                .unwrap_err();
            assert!(
                matches!(
                    error,
                    TypedRegistryGenerationError::Generation(TypedGenerationError::Phase {
                        stage,
                        ..
                    }) if stage == case.stage
                ),
                "{:?}",
                case.stage
            );
            assert_eq!(
                trace.lock().unwrap().as_slice(),
                case.trace,
                "{:?}",
                case.stage
            );
        }
    }

    #[test]
    fn manifest_failure_is_atomic_and_stage_labeled() {
        let (trace, compiler, target) = adapter("program", true);
        let mut registry = TypedBackendRegistry::default();
        registry.register(compiler).unwrap();
        let error = registry
            .compile(&target, &checked_program(), &BackendOptions::default())
            .unwrap_err();
        assert!(matches!(
            error,
            TypedRegistryGenerationError::Generation(TypedGenerationError::Phase {
                stage: TypedPipelineStage::ManifestAssembly,
                ..
            })
        ));
        assert_eq!(trace.lock().unwrap().last(), Some(&"render"));
    }

    #[test]
    fn invalid_options_stop_before_core_lowering() {
        let (trace, compiler, target) = adapter("program", false);
        let mut registry = TypedBackendRegistry::default();
        registry.register(compiler).unwrap();
        let options = BackendOptions::new(BTreeMap::from([(
            "strict".into(),
            OptionValue::Text("wrong".into()),
        )]));
        assert!(matches!(
            registry.compile(&target, &checked_program(), &options),
            Err(TypedRegistryGenerationError::Generation(
                TypedGenerationError::InvalidOptions(_)
            ))
        ));
        assert!(trace.lock().unwrap().is_empty());
    }

    #[test]
    fn registry_rejects_duplicate_and_unknown_targets() {
        let (_, first, target) = adapter("program", false);
        let (_, second, _) = adapter("program", false);
        let mut registry = TypedBackendRegistry::default();
        registry.register(first).unwrap();
        assert_eq!(
            registry.register(second),
            Err(TypedRegistryError::DuplicateTarget(target.clone()))
        );
        let unknown = TargetId::parse("org.polyrust.unknown").unwrap();
        assert!(matches!(
            registry.compile(&unknown, &checked_program(), &BackendOptions::default()),
            Err(TypedRegistryGenerationError::UnknownTarget(value)) if value == unknown
        ));
    }
}
