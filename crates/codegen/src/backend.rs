use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::Arc,
};

use portable_check::v0::{Capability, CheckedProgram};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
use portable_ir::v0::{IrVersion, NodeId};

use crate::OutputManifest;

/// Open, namespaced backend identity such as `org.example.language`.
///
/// ```
/// use portable_codegen::TargetId;
///
/// let target = TargetId::parse("dev.example.my-language")?;
/// assert_eq!(target.as_str(), "dev.example.my-language");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetId(String);

impl TargetId {
    pub fn parse(text: impl Into<String>) -> Result<Self, TargetIdError> {
        let text = text.into();
        let valid_segment = |segment: &str| {
            !segment.is_empty()
                && segment.len() <= 63
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
                && segment.as_bytes()[0].is_ascii_alphanumeric()
                && segment.as_bytes()[segment.len() - 1].is_ascii_alphanumeric()
        };
        if text.len() > 255 || text.split('.').count() < 2 || !text.split('.').all(valid_segment) {
            return Err(TargetIdError { input: text });
        }
        Ok(Self(text))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TargetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetIdError {
    pub input: String,
}

impl fmt::Display for TargetIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid namespaced target ID {:?}", self.input)
    }
}

impl std::error::Error for TargetIdError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BackendVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl BackendVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IrVersionRange {
    pub minimum: IrVersion,
    pub maximum: IrVersion,
}

impl IrVersionRange {
    pub const fn exact(version: IrVersion) -> Self {
        Self {
            minimum: version,
            maximum: version,
        }
    }

    pub const fn contains(self, version: IrVersion) -> bool {
        version.major == self.minimum.major
            && version.major == self.maximum.major
            && version.minor >= self.minimum.minor
            && version.minor <= self.maximum.minor
            && (version.minor != self.minimum.minor || version.patch >= self.minimum.patch)
            && (version.minor != self.maximum.minor || version.patch <= self.maximum.patch)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendDescriptor {
    pub target: TargetId,
    pub display_name: String,
    pub backend_version: BackendVersion,
    pub supported_ir: IrVersionRange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilitySupport {
    Native,
    Helper { helper: String },
    Unsupported { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptionKind {
    Boolean,
    Integer,
    Text,
    Choice(BTreeSet<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionSpec {
    pub kind: OptionKind,
    pub required: bool,
    pub description: String,
}

pub type OptionsSchema = BTreeMap<String, OptionSpec>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptionValue {
    Boolean(bool),
    Integer(i64),
    Text(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BackendOptions(BTreeMap<String, OptionValue>);

impl BackendOptions {
    pub fn new(values: BTreeMap<String, OptionValue>) -> Self {
        Self(values)
    }

    pub fn get(&self, name: &str) -> Option<&OptionValue> {
        self.0.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &OptionValue)> {
        self.0.iter().map(|(name, value)| (name.as_str(), value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendError {
    InvalidOptions(Vec<String>),
    IncompatibleIr {
        actual: IrVersion,
        supported: IrVersionRange,
    },
    UnsupportedCapabilities(Vec<Diagnostic>),
    Generation {
        message: String,
    },
}

/// Extension contract for all source backends.
///
/// Safe callers can only supply a checker-produced [`CheckedProgram`]. The
/// registry validates version/options and performs whole-program capability
/// preflight before calling `generate`.
///
/// Unchecked documents cannot be passed to generation:
///
/// ```compile_fail
/// use portable_codegen::{Backend, BackendOptions};
/// use portable_ir::v0::Document;
///
/// fn bypass(backend: &dyn Backend, unchecked: &Document) {
///     let _ = backend.generate(unchecked, &BackendOptions::default());
/// }
/// ```
pub trait Backend: Send + Sync {
    fn descriptor(&self) -> BackendDescriptor;
    fn support(&self, capability: Capability) -> CapabilitySupport;
    fn options_schema(&self) -> OptionsSchema;
    fn generate(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<OutputManifest, BackendError>;
}

/// Runs deterministic, side-effect-free checks shared by in-tree and external
/// backend implementations. An empty result means the backend honored the
/// descriptor, capability, registry, and generation repeatability contract for
/// the supplied checked program and options.
pub fn check_backend_contract(
    backend: Arc<dyn Backend>,
    program: &CheckedProgram,
    options: &BackendOptions,
) -> Vec<String> {
    let mut violations = Vec::new();
    if backend.descriptor() != backend.descriptor() {
        violations.push("descriptor changed between calls".to_owned());
    }
    if backend.options_schema() != backend.options_schema() {
        violations.push("options schema changed between calls".to_owned());
    }
    for capability in program.capabilities().program() {
        if backend.support(*capability) != backend.support(*capability) {
            violations.push(format!("support for {capability:?} changed between calls"));
        }
    }
    let target = backend.descriptor().target;
    let mut registry = BackendRegistry::default();
    if let Err(error) = registry.register(backend) {
        violations.push(format!("backend could not register: {error:?}"));
        return violations;
    }
    let first = registry.generate(&target, program, options);
    let second = registry.generate(&target, program, options);
    if first != second {
        violations.push("generation changed between identical calls".to_owned());
    }
    violations
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    DuplicateTarget(TargetId),
    UnknownTarget(TargetId),
}

#[derive(Default)]
pub struct BackendRegistry {
    backends: BTreeMap<TargetId, Arc<dyn Backend>>,
}

impl BackendRegistry {
    pub fn register(&mut self, backend: Arc<dyn Backend>) -> Result<(), RegistryError> {
        let target = backend.descriptor().target;
        if self.backends.contains_key(&target) {
            return Err(RegistryError::DuplicateTarget(target));
        }
        self.backends.insert(target, backend);
        Ok(())
    }

    pub fn targets(&self) -> impl Iterator<Item = &TargetId> {
        self.backends.keys()
    }

    pub fn get(&self, target: &TargetId) -> Option<&Arc<dyn Backend>> {
        self.backends.get(target)
    }

    pub fn generate(
        &self,
        target: &TargetId,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<OutputManifest, BackendError> {
        let backend = self.preflight_target(target, program, options)?;
        backend.generate(program, options)
    }

    pub fn preflight_target(
        &self,
        target: &TargetId,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<&Arc<dyn Backend>, BackendError> {
        let backend = self
            .backends
            .get(target)
            .ok_or_else(|| BackendError::Generation {
                message: format!("backend {target} is not registered"),
            })?;
        validate_backend_request(backend.as_ref(), program, options)?;
        Ok(backend)
    }
}

/// Validates a checked program at every public legacy-backend entry point.
///
/// The registry calls this before dispatch, and transitional language plugins
/// call it again from `translate`. The duplicate check is intentional: direct
/// calls to either safe public API must not bypass version, option, or
/// capability diagnostics.
pub fn validate_backend_request(
    backend: &dyn Backend,
    program: &CheckedProgram,
    options: &BackendOptions,
) -> Result<(), BackendError> {
    let descriptor = backend.descriptor();
    let version = program.document().ir_version;
    if !descriptor.supported_ir.contains(version) {
        return Err(BackendError::IncompatibleIr {
            actual: version,
            supported: descriptor.supported_ir,
        });
    }
    let option_errors = validate_options(&backend.options_schema(), options);
    if !option_errors.is_empty() {
        return Err(BackendError::InvalidOptions(option_errors));
    }
    let diagnostics = preflight(backend, program);
    if !diagnostics.is_empty() {
        return Err(BackendError::UnsupportedCapabilities(diagnostics));
    }
    Ok(())
}

pub(crate) fn validate_options(schema: &OptionsSchema, options: &BackendOptions) -> Vec<String> {
    let mut errors = Vec::new();
    for name in schema.keys() {
        if schema[name].required && options.get(name).is_none() {
            errors.push(format!("required option {name:?} is missing"));
        }
    }
    for (name, value) in options.iter() {
        let Some(spec) = schema.get(name) else {
            errors.push(format!("unknown option {name:?}"));
            continue;
        };
        let valid = match (&spec.kind, value) {
            (OptionKind::Boolean, OptionValue::Boolean(_))
            | (OptionKind::Integer, OptionValue::Integer(_))
            | (OptionKind::Text, OptionValue::Text(_)) => true,
            (OptionKind::Choice(values), OptionValue::Text(value)) => values.contains(value),
            _ => false,
        };
        if !valid {
            errors.push(format!("option {name:?} has an invalid value"));
        }
    }
    errors
}

pub fn preflight(backend: &dyn Backend, program: &CheckedProgram) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for capability in program.capabilities().program() {
        if let Some(diagnostic) = unsupported_capability_diagnostic(backend, program, *capability) {
            diagnostics.push(diagnostic);
        }
    }
    portable_diagnostics::sort_diagnostics(&mut diagnostics);
    diagnostics
}

/// Rejects one capability at a transitional backend's direct translation
/// boundary without applying the legacy registry's coarser capability table to
/// otherwise validated feature subsets.
pub fn validate_backend_capability(
    backend: &dyn Backend,
    program: &CheckedProgram,
    capability: Capability,
) -> Result<(), BackendError> {
    match unsupported_capability_diagnostic(backend, program, capability) {
        Some(diagnostic) => Err(BackendError::UnsupportedCapabilities(vec![diagnostic])),
        None => Ok(()),
    }
}

fn unsupported_capability_diagnostic(
    backend: &dyn Backend,
    program: &CheckedProgram,
    capability: Capability,
) -> Option<Diagnostic> {
    if !program.capabilities().program().contains(&capability) {
        return None;
    }
    let CapabilitySupport::Unsupported { reason } = backend.support(capability) else {
        return None;
    };
    let descriptor = backend.descriptor();
    let requiring_nodes: Vec<NodeId> = program
        .capabilities()
        .nodes()
        .filter_map(|(node, capabilities)| capabilities.contains(&capability).then_some(node))
        .collect();
    let source = program
        .module()
        .declarations
        .iter()
        .find(|declaration| {
            program
                .capabilities()
                .declaration(declaration.header().node.id)
                .is_some_and(|capabilities| capabilities.contains(&capability))
        })
        .map_or_else(
            || SourceRef::logical([format!("module({})", program.module().name)]),
            |declaration| declaration.header().node.source.clone(),
        );
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::UnsupportedCapability,
        format!(
            "backend {} does not support {capability:?}: {reason}",
            descriptor.target
        ),
        source,
    );
    diagnostic.target = Some(descriptor.target.to_string());
    diagnostic.notes.push(format!(
        "requiring node IDs: {}",
        requiring_nodes
            .iter()
            .map(|node| node.0.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    Some(diagnostic)
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use portable_ir::v0::{
        Declaration, DeclarationHeader, Document, IrVersion, MemberHeader, Module, NodeId,
        NodeMeta, RecordDeclaration, SourceRef, TypeRef, Visibility,
    };

    use super::*;
    use crate::OutputFile;

    struct MockBackend {
        target: TargetId,
        supported_ir: IrVersionRange,
        unicode: CapabilitySupport,
        fail: bool,
        calls: Arc<AtomicUsize>,
    }

    impl MockBackend {
        fn new(id: &str, calls: Arc<AtomicUsize>) -> Self {
            Self {
                target: TargetId::parse(id).unwrap(),
                supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
                unicode: CapabilitySupport::Native,
                fail: false,
                calls,
            }
        }
    }

    impl Backend for MockBackend {
        fn descriptor(&self) -> BackendDescriptor {
            BackendDescriptor {
                target: self.target.clone(),
                display_name: "Backend mock".to_owned(),
                backend_version: BackendVersion::new(0, 1, 0),
                supported_ir: self.supported_ir,
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
                    description: "Output style".to_owned(),
                },
            )])
        }

        fn generate(
            &self,
            program: &CheckedProgram,
            _options: &BackendOptions,
        ) -> Result<OutputManifest, BackendError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                return Err(BackendError::Generation {
                    message: "deliberate mock failure".to_owned(),
                });
            }
            OutputManifest::new(
                vec![OutputFile::text(
                    "src/generated.txt",
                    format!("module={}\n", program.module().name),
                )],
                vec![],
                vec![],
            )
            .map_err(BackendError::UnsupportedCapabilities)
        }
    }

    fn checked_unicode_program() -> CheckedProgram {
        let source = SourceRef::logical(["module(contract_test)", "record(Message)"]);
        let document = Document::new(
            IrVersion::CURRENT,
            Module {
                name: "contract_test".to_owned(),
                declarations: vec![Declaration::Record(RecordDeclaration {
                    header: DeclarationHeader {
                        node: NodeMeta::new(NodeId(1), source.clone()),
                        name: "Message".to_owned(),
                        visibility: Visibility::Public,
                        documentation: vec![],
                    },
                    fields: vec![portable_ir::v0::FieldDeclaration {
                        header: MemberHeader {
                            node: NodeMeta::new(NodeId(2), source),
                            name: "text".to_owned(),
                            documentation: vec![],
                        },
                        ty: TypeRef::String,
                    }],
                })],
            },
        );
        portable_check::v0::check_program(document).unwrap()
    }

    #[test]
    fn target_ids_are_open_namespaced_text() {
        assert_eq!(
            TargetId::parse("dev.example.experimental-language")
                .unwrap()
                .as_str(),
            "dev.example.experimental-language"
        );
        for invalid in ["rust", "Org.example.rust", "org..rust", "org.example.rust_"] {
            assert!(TargetId::parse(invalid).is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn registry_accepts_new_targets_and_rejects_duplicates() {
        let calls = Arc::new(AtomicUsize::new(0));
        let backend: Arc<dyn Backend> = Arc::new(MockBackend::new(
            "dev.external.unmodified-core",
            calls.clone(),
        ));
        let target = backend.descriptor().target;
        let mut registry = BackendRegistry::default();
        registry.register(backend).unwrap();
        assert!(registry.get(&target).is_some());
        assert_eq!(registry.targets().collect::<Vec<_>>(), vec![&target]);
        assert_eq!(
            registry.register(Arc::new(MockBackend::new(
                "dev.external.unmodified-core",
                calls
            ))),
            Err(RegistryError::DuplicateTarget(target))
        );
    }

    #[test]
    fn preflight_reports_every_unsupported_capability_before_generation() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut backend = MockBackend::new("org.polyrust.mock", calls.clone());
        backend.unicode = CapabilitySupport::Unsupported {
            reason: "test rejection".to_owned(),
        };
        let target = backend.target.clone();
        let mut registry = BackendRegistry::default();
        registry.register(Arc::new(backend)).unwrap();
        let error = registry
            .generate(
                &target,
                &checked_unicode_program(),
                &BackendOptions::default(),
            )
            .unwrap_err();
        let BackendError::UnsupportedCapabilities(diagnostics) = error else {
            panic!("unexpected backend error")
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, DiagnosticCode::UnsupportedCapability);
        assert_eq!(diagnostics[0].target.as_deref(), Some("org.polyrust.mock"));
        assert!(diagnostics[0].notes[0].contains("2"));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn invalid_options_and_incompatible_ir_never_call_generate() {
        let program = checked_unicode_program();
        let calls = Arc::new(AtomicUsize::new(0));
        let target = TargetId::parse("org.polyrust.mock").unwrap();
        let mut registry = BackendRegistry::default();
        registry
            .register(Arc::new(MockBackend::new(
                "org.polyrust.mock",
                calls.clone(),
            )))
            .unwrap();
        let options = BackendOptions::new(BTreeMap::from([(
            "style".to_owned(),
            OptionValue::Text("invalid".to_owned()),
        )]));
        assert!(matches!(
            registry.generate(&target, &program, &options),
            Err(BackendError::InvalidOptions(_))
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        let mut incompatible = MockBackend::new("org.polyrust.old", calls.clone());
        incompatible.supported_ir = IrVersionRange::exact(IrVersion::new(0, 0, 0));
        let old_target = incompatible.target.clone();
        registry.register(Arc::new(incompatible)).unwrap();
        assert!(matches!(
            registry.generate(&old_target, &program, &BackendOptions::default()),
            Err(BackendError::IncompatibleIr { .. })
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn interface_kit_proves_repeatability_and_preserves_backend_errors() {
        let program = checked_unicode_program();
        let calls = Arc::new(AtomicUsize::new(0));
        let backend: Arc<dyn Backend> =
            Arc::new(MockBackend::new("org.polyrust.repeatable", calls.clone()));
        assert!(check_backend_contract(backend, &program, &BackendOptions::default()).is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 2);

        let mut failing = MockBackend::new("org.polyrust.failing", calls);
        failing.fail = true;
        let target = failing.target.clone();
        let mut registry = BackendRegistry::default();
        registry.register(Arc::new(failing)).unwrap();
        assert_eq!(
            registry.generate(&target, &program, &BackendOptions::default()),
            Err(BackendError::Generation {
                message: "deliberate mock failure".to_owned()
            })
        );
    }
}
