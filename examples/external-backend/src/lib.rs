#![forbid(unsafe_code)]

use std::{collections::BTreeMap, sync::Arc};

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendRegistry, BackendVersion,
    CapabilitySupport, Document, FileGroup, FileGroupId, ImportGroup, ImportSet, IrVersionRange,
    LanguageFile, LanguageFragment, LanguagePackage, LanguagePlugin, LanguageRenderer,
    LanguageSourceFile, OptionsSchema, OutputManifest, RawText, RuntimeHelper, RuntimeHelperGraph,
    SourceFileRole, TargetId, check_backend_contract, generate_with_plugin, preflight,
};
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
