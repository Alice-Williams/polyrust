#![forbid(unsafe_code)]

use std::{collections::BTreeMap, sync::Arc};

use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendRegistry, BackendVersion,
    CapabilitySupport, IrVersionRange, OptionsSchema, OutputFile, OutputManifest, TargetId,
    check_backend_contract, preflight,
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
        _options: &BackendOptions,
    ) -> Result<OutputManifest, BackendError> {
        OutputManifest::new(
            vec![OutputFile::text(
                "SUMMARY.txt",
                format!(
                    "module={}\ndeclarations={}\n",
                    program.module().name,
                    program.module().declarations.len()
                ),
            )],
            vec![],
            vec![],
        )
        .map_err(BackendError::UnsupportedCapabilities)
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
        &portable_codegen::OutputContents::Text("module=external_example\ndeclarations=0\n".into())
    );
}
