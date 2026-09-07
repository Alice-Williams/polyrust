use crate::ast::{JavaExpr, JavaStmt, JavaType, JavaVisibility};
use crate::capabilities::JavaCapabilitySet;
use crate::dialect::JavaDialect;
use crate::preflight::JavaCapabilitySelection;
pub(crate) use call_builders::{
    known_call, known_generic_call, member_call, runtime_call, runtime_fallible,
};
pub(crate) use declaration_builders::{identifier, private_constructor, visibility_modifier};
pub(crate) use expression_builders::{
    binary, bool_literal, conditional, f64_literal, i32_literal, i64_literal, java_unit_value,
    scalar_literal, string_literal, unary,
};
use portable_codegen::{
    BackendOptions, FileGroupRole, GeneratedCallableId, GeneratedInterfaceMethodId,
    GeneratedSymbolId, GeneratedTypeId, GeneratedValueId, RelativeOutputPath, TargetArtifact,
    TargetAstBuilder, TargetAstPackage, TargetFileGroup, TargetFileMember, TargetLowerer,
    VerifiedCapabilities, VerifiedCore,
};
use portable_core_ir::{
    CoreConstantId, CoreEnumId, CoreFunctionId, CoreInterfaceId, CoreInterfaceMethodId,
    CoreProgram, CoreRecordId, CoreVariantId,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
use portable_ir::v0::Visibility;
use std::cell::Cell;
use std::collections::BTreeMap;
mod blocks;
mod boundary;
mod call_builders;
mod callable_declarations;
mod constant_dependencies;
mod constant_order;
mod constant_types;
mod constants;
mod declaration_builders;
mod enums;
mod evaluation_plans;
mod expression_builders;
mod expressions;
mod generated_file;
mod interfaces;
mod intrinsic_mapping;
mod intrinsic_plans;
mod mapping_nodes;
mod native_test_file;
mod patterns;
mod portable_tests;
mod record_members;
mod records;
mod registration;
mod support_files;
mod types;
mod values;

#[derive(Clone, Copy, Debug)]
pub struct JavaLowerer {
    features: JavaCapabilitySet,
}

impl JavaLowerer {
    pub(crate) const fn new(features: JavaCapabilitySet) -> Self {
        Self { features }
    }
}

impl TargetLowerer<CoreProgram, JavaDialect> for JavaLowerer {
    type Capabilities = JavaCapabilitySelection;

    fn lower_target(
        &self,
        core: &VerifiedCore<CoreProgram>,
        capabilities: &VerifiedCapabilities<Self::Capabilities>,
        _options: &BackendOptions,
    ) -> Result<TargetAstPackage<JavaDialect>, Vec<Diagnostic>> {
        Lowering::new(core.value(), capabilities.selection(), self.features).lower()
    }
}

struct Lowering<'a> {
    core: &'a CoreProgram,
    builder: TargetAstBuilder<JavaDialect>,
    declared: Vec<GeneratedSymbolId>,
    entry: Option<GeneratedTypeId>,
    records: BTreeMap<CoreRecordId, GeneratedTypeId>,
    enums: BTreeMap<CoreEnumId, GeneratedTypeId>,
    variants: BTreeMap<CoreVariantId, GeneratedTypeId>,
    enum_values: BTreeMap<CoreVariantId, GeneratedValueId>,
    interfaces: BTreeMap<CoreInterfaceId, GeneratedTypeId>,
    functions: BTreeMap<CoreFunctionId, GeneratedCallableId>,
    interface_methods: BTreeMap<CoreInterfaceMethodId, GeneratedInterfaceMethodId>,
    constants: BTreeMap<CoreConstantId, GeneratedValueId>,
    capabilities: &'a JavaCapabilitySelection,
    features: JavaCapabilitySet,
    next_temporary: Cell<u32>,
}

impl<'a> Lowering<'a> {
    fn new(
        core: &'a CoreProgram,
        capabilities: &'a JavaCapabilitySelection,
        features: JavaCapabilitySet,
    ) -> Self {
        Self {
            core,
            builder: TargetAstBuilder::new(JavaDialect),
            declared: vec![],
            entry: None,
            records: BTreeMap::new(),
            enums: BTreeMap::new(),
            variants: BTreeMap::new(),
            enum_values: BTreeMap::new(),
            interfaces: BTreeMap::new(),
            functions: BTreeMap::new(),
            interface_methods: BTreeMap::new(),
            constants: BTreeMap::new(),
            capabilities,
            features,
            next_temporary: Cell::new(0),
        }
    }

    fn lower(mut self) -> Result<TargetAstPackage<JavaDialect>, Vec<Diagnostic>> {
        self.capabilities.validate_for(self.core)?;
        self.register_types();
        self.register_values_and_callables()?;
        let generated = self.generated_file()?;
        let runtime = self.runtime_file()?;
        let conformance = self.conformance_file()?;
        let native_test = self.native_test_file()?;
        let negative = self.negative_file()?;
        let readme = self.builder.artifact(TargetArtifact::Documentation {
            path: path("README.md"),
            contents: "# Generated PolyRust Java package\n\nCompile with Java 21 or newer. The package has no third-party runtime dependencies.\n".to_owned(),
            source: source("documentation"),
        });
        for (role, file, label) in [
            (FileGroupRole::PublicApi, generated, "source-group"),
            (FileGroupRole::Runtime, runtime, "runtime-group"),
            (FileGroupRole::NativeTests, native_test, "native-test-group"),
            (FileGroupRole::Conformance, conformance, "conformance-group"),
            (FileGroupRole::NegativeTests, negative, "negative-group"),
        ] {
            self.builder.group(TargetFileGroup::new(
                role,
                vec![TargetFileMember::Source(file)],
                source(label),
            ));
        }
        self.builder.group(TargetFileGroup::new(
            FileGroupRole::Documentation,
            vec![TargetFileMember::Artifact(readme)],
            source("documentation-group"),
        ));
        Ok(self.builder.build())
    }
}

pub(crate) fn path(value: &str) -> RelativeOutputPath {
    RelativeOutputPath::new(value).expect("static Java output path is safe")
}

pub(crate) fn source(value: &str) -> SourceRef {
    SourceRef::logical(["java-lowering", value])
}

pub(crate) fn diagnostic(message: &str) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::InvalidStructure, message, source("error"))
}

pub(crate) fn java_visibility(value: Visibility) -> JavaVisibility {
    match value {
        Visibility::Public => JavaVisibility::Public,
        Visibility::Package => JavaVisibility::Private,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BlockMode {
    ReturnResult,
    AssignResult { target: Box<JavaExpr> },
    StatementBody,
}

struct ExprPlan {
    statements: Vec<JavaStmt>,
    value: JavaExpr,
}

impl ExprPlan {
    fn pure(value: JavaExpr) -> Self {
        Self {
            statements: vec![],
            value,
        }
    }
}

#[doc(hidden)]
pub enum JavaIntrinsicExpr {
    Direct(JavaExpr),
    Fallible {
        call: JavaExpr,
        value_type: JavaType,
    },
}
