#![forbid(unsafe_code)]

use std::{collections::BTreeMap, sync::Arc};

use portable_backend_java::{
    ast::{JavaFilePlacement, JavaPackage, JavaSourceFileKind},
    dialect::JavaDialect,
};
use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendRegistry, BackendVersion,
    CapabilitySupport, CertifiedSourceFile, CertifiedStructuralRendererAdapter, CoreLowerer,
    Document, FileGroup, FileGroupId, FileGroupRole, ImportGroup, ImportSet, IrVersionRange,
    LanguageFile, LanguageFragment, LanguagePackage, LanguagePlugin, LanguageRenderer,
    LanguageSourceFile, OptionsSchema, OutputManifest, RawText, RelativeOutputPath, RuntimeHelper,
    RuntimeHelperGraph, SourceFileRole, SourceRole, TargetAstBuilder, TargetCapabilityRegistry,
    TargetFile, TargetFileGroup, TargetFileMember, TargetId, TargetLinker, TargetLowerer,
    TotalSourceRenderer, TypedCompiler, TypedCompilerAdapter, TypedLanguagePlugin,
    VerifiedCapabilities, VerifiedCore, check_backend_contract, generate_with_plugin, preflight,
};
use portable_diagnostics::{Diagnostic, SourceRef};
use portable_ir::v0::IrVersion;

struct TextSummaryBackend;

impl Backend for TextSummaryBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("dev.example.text-summary").expect("static target ID"),
            display_name: "External text summary".into(),
            backend_version: BackendVersion::new(0, 1, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }

    fn support(&self, _capability: Capability) -> CapabilitySupport {
        CapabilitySupport::Native
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
enum SummaryImport {
    CheckedModule,
}

struct SummaryRenderer;

impl LanguageRenderer<SummaryImport> for SummaryRenderer {
    fn render_imports(&self, imports: &ImportSet<SummaryImport>) -> Result<Document, String> {
        let lines = imports
            .groups()
            .flat_map(|(_, group)| group.iter())
            .map(|import| match import {
                SummaryImport::CheckedModule => "requires checked-module",
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(Document::raw_text(RawText::new(lines)))
    }
}

impl LanguagePlugin for TextSummaryBackend {
    type Import = SummaryImport;
    type Renderer = SummaryRenderer;

    fn translate(
        &self,
        program: &CheckedProgram,
        _options: &BackendOptions,
    ) -> Result<LanguagePackage<Self::Import>, BackendError> {
        let import_group = ImportGroup::new(10, "checked-input").map_err(generation_error)?;
        let summary = LanguageFragment::new(Document::raw_text(RawText::new(format!(
            "module={}\ndeclarations={}",
            program.module().name,
            program.module().declarations.len()
        ))))
        .with_import(import_group, SummaryImport::CheckedModule)
        .with_helper_root("runtime.format");

        let helpers = RuntimeHelperGraph::new([RuntimeHelper::new(
            "runtime.format",
            0,
            LanguageFragment::new(Document::raw_text(RawText::new(
                "format=polyrust-summary-v0\n",
            ))),
        )])
        .map_err(generation_error)?;
        let runtime = helpers
            .resolve(summary.helper_roots())
            .map_err(generation_error)?;

        let mut source = LanguageSourceFile::new("SUMMARY.txt", SourceFileRole::Source);
        source.set_body(LanguageFragment::sequence([runtime, summary]));
        let group = FileGroup::new(
            FileGroupId::parse("source").map_err(generation_error)?,
            vec![LanguageFile::source(source)],
        )
        .map_err(generation_error)?;
        LanguagePackage::new(vec![group], vec![], vec![]).map_err(generation_error)
    }

    fn renderer(&self) -> Self::Renderer {
        SummaryRenderer
    }
}

fn generation_error(error: impl ToString) -> BackendError {
    BackendError::Generation {
        message: error.to_string(),
    }
}

#[test]
fn external_backend_registers_preflights_generates_and_passes_contract() {
    let program = portable_build::ModuleBuilder::new("external_example")
        .finish()
        .expect("empty module checks");
    let backend: Arc<dyn Backend> = Arc::new(TextSummaryBackend);
    assert!(preflight(backend.as_ref(), &program).is_empty());
    assert!(
        check_backend_contract(backend.clone(), &program, &BackendOptions::default()).is_empty()
    );

    let target = backend.descriptor().target;
    let mut registry = BackendRegistry::default();
    registry
        .register(backend)
        .expect("external target registers");
    let manifest = registry
        .generate(&target, &program, &BackendOptions::default())
        .expect("external target generates");
    assert_eq!(
        manifest
            .file("SUMMARY.txt")
            .expect("summary artifact")
            .contents(),
        &portable_codegen::OutputContents::Text(
            "requires checked-module\n\nformat=polyrust-summary-v0\nmodule=external_example\ndeclarations=0\n"
                .into()
        )
    );
}

#[derive(Clone, Copy)]
struct ExternalCoreLowerer;

impl CoreLowerer for ExternalCoreLowerer {
    type Core = ();

    fn lower_core(&self, _program: &CheckedProgram) -> Result<Self::Core, Vec<Diagnostic>> {
        Ok(())
    }

    fn verify_core(&self, _core: &Self::Core) -> Result<(), Vec<Diagnostic>> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ExternalCapabilities;

impl TargetCapabilityRegistry<()> for ExternalCapabilities {
    type Selection = ();

    fn preflight(&self, _core: &VerifiedCore<()>) -> Result<Self::Selection, Vec<Diagnostic>> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ExternalJavaLowerer;

impl TargetLowerer<(), JavaDialect> for ExternalJavaLowerer {
    type Capabilities = ();

    fn lower_target(
        &self,
        _core: &VerifiedCore<()>,
        _capabilities: &VerifiedCapabilities<()>,
        _options: &BackendOptions,
    ) -> Result<portable_codegen::TargetAstPackage<JavaDialect>, Vec<Diagnostic>> {
        let source = SourceRef::logical(["external-certified-plugin", "empty-java-unit"]);
        let mut builder = TargetAstBuilder::new(JavaDialect);
        let file = builder.file(TargetFile::new(
            RelativeOutputPath::new(
                "src/main/java/org/polyrust/generated/ExternalCertificate.java",
            )
            .expect("static Java path is valid"),
            SourceRole::Implementation,
            JavaPackage::Generated,
            JavaFilePlacement::Main,
            vec![],
            JavaSourceFileKind::CompilationUnit,
            source.clone(),
        ));
        builder.group(TargetFileGroup::new(
            FileGroupRole::Implementation,
            vec![TargetFileMember::Source(file)],
            source,
        ));
        Ok(builder.build())
    }
}

#[derive(Clone, Copy)]
struct ExternalEmptyJavaRenderer;

impl TotalSourceRenderer<JavaDialect> for ExternalEmptyJavaRenderer {
    fn target_name(&self) -> &'static str {
        "dev.example.certified-empty-java"
    }

    fn render_file(&self, file: CertifiedSourceFile<'_, JavaDialect>) -> String {
        assert_eq!(
            file.file().path().as_str(),
            "src/main/java/org/polyrust/generated/ExternalCertificate.java"
        );
        String::new()
    }
}

#[derive(Clone, Copy)]
struct ExternalCertifiedJavaPlugin;

impl TypedLanguagePlugin<()> for ExternalCertifiedJavaPlugin {
    type Dialect = JavaDialect;
    type CapabilityRegistry = ExternalCapabilities;
    type Lowerer = ExternalJavaLowerer;
    type Resolver = TargetLinker<JavaDialect>;
    type Renderer = CertifiedStructuralRendererAdapter<JavaDialect, ExternalEmptyJavaRenderer>;

    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("dev.example.certified-empty-java")
                .expect("static target ID is valid"),
            display_name: "External certified Java".into(),
            backend_version: BackendVersion::new(0, 1, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }

    fn options_schema(&self) -> OptionsSchema {
        BTreeMap::new()
    }

    fn dialect(&self) -> Self::Dialect {
        JavaDialect
    }

    fn capability_registry(&self) -> Self::CapabilityRegistry {
        ExternalCapabilities
    }

    fn lowerer(&self) -> Self::Lowerer {
        ExternalJavaLowerer
    }

    fn resolver(&self) -> Self::Resolver {
        TargetLinker::new(JavaDialect)
    }

    fn renderer(&self) -> Self::Renderer {
        CertifiedStructuralRendererAdapter::new(ExternalEmptyJavaRenderer)
    }
}

#[test]
fn external_typed_plugin_uses_the_sealed_certificate_sequence() {
    let program = portable_build::ModuleBuilder::new("external_certified_example")
        .finish()
        .expect("empty module checks");
    let compiler = TypedCompilerAdapter::new(ExternalCoreLowerer, ExternalCertifiedJavaPlugin);
    let manifest = compiler
        .compile_checked(&program, &BackendOptions::default())
        .expect("external typed plugin traverses verification, linking, and certification");
    assert_eq!(manifest.files().len(), 1);
    assert_eq!(
        manifest
            .file("src/main/java/org/polyrust/generated/ExternalCertificate.java")
            .expect("certified source file")
            .contents(),
        &portable_codegen::OutputContents::Text("\n".into())
    );
}
