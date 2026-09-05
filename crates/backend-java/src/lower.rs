use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
};

use portable_build::{
    BooleanLogic, BytesOperations, CapabilityMapping, CheckedIntegerArithmetic,
    CheckedIntegerShifts, Equality, FloatingPointArithmetic, FloatingPointInspection,
    IntegerBitwise, IntegerConversions, ListOperations, OptionOperations, Ordering,
    ResultOperations, StringConcatenation, StringInspection, StringTransformation, Utf8Conversions,
    WrappingIntegerArithmetic,
};
use portable_codegen::{
    BackendOptions, FileGroupRole, GeneratedCallable, GeneratedCallableId,
    GeneratedInterfaceMethod, GeneratedInterfaceMethodId, GeneratedOrigin, GeneratedSymbolId,
    GeneratedType, GeneratedTypeId, GeneratedValue, GeneratedValueId, RelativeOutputPath,
    SourceRole, SynthesisReason, TargetArtifact, TargetAstBuilder, TargetAstPackage,
    TargetCallableSignature, TargetFile, TargetFileGroup, TargetFileMember, TargetLowerer,
    VerifiedCapabilities, VerifiedCore,
};
use portable_core_ir::{
    CoreBinaryIntrinsic, CoreBlockId, CoreConstantExpr, CoreConstantExprKind, CoreConstantId,
    CoreDeclaration, CoreEnumId, CoreExprField, CoreExprId, CoreExprKind, CoreFieldId,
    CoreFunctionId, CoreImplementationMethodId, CoreInterfaceId, CoreInterfaceMethodId,
    CoreIntrinsicExpr, CoreMatchArm, CorePattern, CoreProgram, CoreRecordId, CoreStatement,
    CoreTernaryIntrinsic, CoreTestInvocation, CoreType, CoreTypeId, CoreTypedValue,
    CoreUnaryIntrinsic, CoreValue, CoreValueField, CoreVariadicIntrinsic, CoreVariantId,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
use portable_ir::v0::Visibility;

use crate::{
    ast::*,
    capabilities::{
        JavaBytesInput, JavaCapabilitySet, JavaConcreteInterfaceCallInput,
        JavaConditionalValueInput, JavaConditionalsInput, JavaConditionalsNode, JavaConstantsInput,
        JavaConstantsNode, JavaEnumBranchInput, JavaEnumVariantInput, JavaEnumsInput,
        JavaEnumsNode, JavaFunctionDeclarationInput, JavaFunctionsInput, JavaFunctionsNode,
        JavaInterfaceCallInput, JavaInterfaceDeclarationInput, JavaInterfaceImplementationInput,
        JavaInterfaceMethodInput, JavaInterfacesInput, JavaInterfacesNode, JavaIntrinsicFamily,
        JavaListInput, JavaLocalBindingInput, JavaLoopsInput, JavaLoweredPattern,
        JavaMatchArmInput, JavaMatchInput, JavaModuleInput, JavaOptionInput,
        JavaPatternFieldBindingInput, JavaPatternInput, JavaPatternMatchPlan,
        JavaPatternMatchingInput, JavaPatternMatchingNode, JavaPortableTestCaseInput,
        JavaPortableTestExpectation, JavaPortableTestHarnessInput, JavaPortableTestsInput,
        JavaPortableTestsNode, JavaRecordDeclarationInput, JavaRecordsInput, JavaRecordsNode,
        JavaResultInput, JavaResultPropagationInput, JavaResultPropagationPlan, JavaTypeAliasInput,
        classify_intrinsic,
    },
    capability::JavaCapabilitySelection,
    dialect::*,
};

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

    fn register_types(&mut self) {
        let entry = self.builder.generated_type(GeneratedType {
            name: "Generated".to_owned(),
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
            source: source("Generated"),
        });
        self.declared.push(GeneratedSymbolId::Type(entry));
        self.entry = Some(entry);
        for declaration in &self.core.module().declarations {
            match *declaration {
                CoreDeclaration::Record(id) => {
                    let item = self.core.record(id).expect("verified record");
                    let generated = self.builder.generated_type(GeneratedType {
                        name: item.header.name.clone(),
                        kind: JavaDeclarationKind::Record,
                        visibility: java_visibility(item.header.visibility),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: item.header.source.clone(),
                    });
                    self.records.insert(id, generated);
                    self.declared.push(GeneratedSymbolId::Type(generated));
                }
                CoreDeclaration::Enum(id) => {
                    let item = self.core.enumeration(id).expect("verified enum");
                    let payload_free = self.enum_is_payload_free(id);
                    let generated = self.builder.generated_type(GeneratedType {
                        name: item.header.name.clone(),
                        kind: if payload_free {
                            JavaDeclarationKind::Enum
                        } else {
                            JavaDeclarationKind::SealedInterface
                        },
                        visibility: java_visibility(item.header.visibility),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: item.header.source.clone(),
                    });
                    self.enums.insert(id, generated);
                    self.declared.push(GeneratedSymbolId::Type(generated));
                    for variant in item.variants.iter().filter(|_| !payload_free) {
                        let value = self.core.variant(*variant).expect("verified variant");
                        let generated = self.builder.generated_type(GeneratedType {
                            name: format!("{}{}", item.header.name, value.header.name),
                            kind: JavaDeclarationKind::Record,
                            visibility: java_visibility(item.header.visibility),
                            origin: GeneratedOrigin::CoreDeclaration(*declaration),
                            source: value.header.source.clone(),
                        });
                        self.variants.insert(*variant, generated);
                        self.declared.push(GeneratedSymbolId::Type(generated));
                    }
                }
                CoreDeclaration::Interface(id) => {
                    let item = self.core.interface(id).expect("verified interface");
                    let generated = self.builder.generated_type(GeneratedType {
                        name: item.header.name.clone(),
                        kind: JavaDeclarationKind::SealedInterface,
                        visibility: java_visibility(item.header.visibility),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: item.header.source.clone(),
                    });
                    self.interfaces.insert(id, generated);
                    self.declared.push(GeneratedSymbolId::Type(generated));
                }
                CoreDeclaration::Constant(_)
                | CoreDeclaration::Alias(_)
                | CoreDeclaration::Implementation(_)
                | CoreDeclaration::Function(_)
                | CoreDeclaration::Test(_) => {}
            }
        }
    }

    fn register_values_and_callables(&mut self) -> Result<(), Vec<Diagnostic>> {
        for declaration in &self.core.module().declarations {
            match *declaration {
                CoreDeclaration::Constant(id) => {
                    let value = self.core.constant(id).expect("verified constant");
                    let java_type = self.ty(value.ty)?;
                    let symbol = self.builder.value(GeneratedValue {
                        name: value.header.name.clone(),
                        ty: JavaDialect.registered_type(&java_type),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: value.header.source.clone(),
                    });
                    self.constants.insert(id, symbol);
                    self.declared.push(GeneratedSymbolId::Value(symbol));
                }
                CoreDeclaration::Function(id) => {
                    let value = self.core.function(id).expect("verified function");
                    let parameters = value
                        .parameters
                        .iter()
                        .map(|parameter| {
                            self.ty(parameter.ty)
                                .map(|ty| JavaDialect.registered_type(&ty))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = self.poly_result_type(value.return_type)?;
                    let symbol = self.builder.callable(GeneratedCallable {
                        name: value.header.name.clone(),
                        signature: TargetCallableSignature {
                            invocation: JavaInvocationKind::Static,
                            receiver: None,
                            parameters,
                            return_type: JavaDialect.registered_type(&result),
                        },
                        visibility: java_visibility(value.header.visibility),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: value.header.source.clone(),
                    });
                    self.functions.insert(id, symbol);
                    self.declared.push(GeneratedSymbolId::Callable(symbol));
                }
                CoreDeclaration::Interface(id) => {
                    let interface = self.core.interface(id).expect("verified interface");
                    let owner = self.interfaces[&id];
                    for method_id in &interface.methods {
                        let method = self
                            .core
                            .interface_method(*method_id)
                            .expect("verified method");
                        let parameters = method
                            .parameters
                            .iter()
                            .map(|parameter| {
                                self.ty(parameter.ty)
                                    .map(|ty| JavaDialect.registered_type(&ty))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let receiver = JavaType::Reference(JavaTypeName::Generated(owner));
                        let result = self.poly_result_type(method.return_type)?;
                        let symbol = self.builder.interface_method(GeneratedInterfaceMethod {
                            owner,
                            name: method.header.name.clone(),
                            signature: TargetCallableSignature {
                                invocation: JavaInvocationKind::Instance,
                                receiver: Some(JavaDialect.registered_type(&receiver)),
                                parameters,
                                return_type: JavaDialect.registered_type(&result),
                            },
                            origin: GeneratedOrigin::CoreDeclaration(*declaration),
                            source: method.header.source.clone(),
                        });
                        self.interface_methods.insert(*method_id, symbol);
                        self.declared
                            .push(GeneratedSymbolId::InterfaceMethod(symbol));
                    }
                }
                CoreDeclaration::Alias(_)
                | CoreDeclaration::Record(_)
                | CoreDeclaration::Implementation(_)
                | CoreDeclaration::Test(_) => {}
                CoreDeclaration::Enum(id) => {
                    if self.enum_is_payload_free(id) {
                        let enumeration = self.core.enumeration(id).expect("verified enum");
                        let ty = JavaType::Reference(JavaTypeName::Generated(self.enums[&id]));
                        for variant_id in &enumeration.variants {
                            let variant = self.core.variant(*variant_id).expect("verified variant");
                            let symbol = self.builder.value(GeneratedValue {
                                name: variant.header.name.clone(),
                                ty: JavaDialect.registered_type(&ty),
                                origin: GeneratedOrigin::CoreDeclaration(*declaration),
                                source: variant.header.source.clone(),
                            });
                            self.enum_values.insert(*variant_id, symbol);
                            self.declared.push(GeneratedSymbolId::Value(symbol));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn generated_file(&mut self) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let mut members = Vec::new();
        for id in self.ordered_constant_ids()? {
            members.push(JavaMember::Field(self.constant_field(id)?));
        }
        for declaration in &self.core.module().declarations {
            match *declaration {
                CoreDeclaration::Record(id) => {
                    members.push(JavaMember::NestedType(self.record_declaration(id)?));
                }
                CoreDeclaration::Enum(id) => {
                    members.extend(
                        self.enum_declarations(id)?
                            .into_iter()
                            .map(JavaMember::NestedType),
                    );
                }
                CoreDeclaration::Interface(id) => {
                    members.push(JavaMember::NestedType(self.interface_declaration(id)?));
                }
                CoreDeclaration::Function(id) => {
                    members.push(JavaMember::Method(self.function_method(id)?))
                }
                CoreDeclaration::Alias(id) => {
                    let alias = self.core.alias(id).expect("verified alias");
                    let _erased = self
                        .features
                        .mapping_for::<portable_build::TypeAliases>()
                        .lower(
                            &mut (),
                            JavaTypeAliasInput {
                                name: alias.header.name.clone(),
                                target: self.ty(alias.target)?,
                            },
                        )?;
                }
                CoreDeclaration::Constant(_)
                | CoreDeclaration::Implementation(_)
                | CoreDeclaration::Test(_) => {}
            }
        }
        members.extend(self.public_tagged_value_factories()?);
        let file = self
            .features
            .mapping_for::<portable_build::Modules>()
            .lower(
                &mut (),
                JavaModuleInput {
                    entry: self.entry.expect("Java module entry registered"),
                    declared: self.declared.clone(),
                    members,
                },
            )?;
        Ok(self.builder.file(file))
    }

    fn ordered_constant_ids(&self) -> Result<Vec<CoreConstantId>, Vec<Diagnostic>> {
        let mut ordered = Vec::new();
        let mut visiting = BTreeSet::new();
        let mut emitted = BTreeSet::new();
        for declaration in &self.core.module().declarations {
            if let CoreDeclaration::Constant(id) = declaration {
                self.visit_constant(*id, &mut visiting, &mut emitted, &mut ordered)?;
            }
        }
        Ok(ordered)
    }

    fn visit_constant(
        &self,
        id: CoreConstantId,
        visiting: &mut BTreeSet<CoreConstantId>,
        emitted: &mut BTreeSet<CoreConstantId>,
        ordered: &mut Vec<CoreConstantId>,
    ) -> Result<(), Vec<Diagnostic>> {
        if emitted.contains(&id) {
            return Ok(());
        }
        if !visiting.insert(id) {
            return Err(vec![diagnostic(
                "verified CoreIR contains a cyclic Java constant dependency",
            )]);
        }
        let constant = self
            .core
            .constant(id)
            .ok_or_else(|| vec![diagnostic("missing CoreIR constant dependency")])?;
        let mut dependencies = BTreeSet::new();
        collect_constant_dependencies(&constant.value, &mut dependencies);
        for dependency in dependencies {
            self.visit_constant(dependency, visiting, emitted, ordered)?;
        }
        visiting.remove(&id);
        emitted.insert(id);
        ordered.push(id);
        Ok(())
    }

    fn runtime_file(&mut self) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        Ok(self.builder.file(TargetFile::new(
            path("src/main/java/org/polyrust/generated/Runtime.java"),
            SourceRole::Runtime,
            JavaPackage::Generated,
            JavaFilePlacement::Runtime,
            vec![crate::runtime::shell_item()],
            JavaSourceFileKind::CompilationUnit,
            source("runtime-file"),
        )))
    }

    fn conformance_file(&mut self) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let declaration = self.test_declaration("ConformanceTest")?;
        Ok(self.builder.file(TargetFile::new(
            path("src/test/java/org/polyrust/generated/ConformanceTest.java"),
            SourceRole::Conformance,
            JavaPackage::Generated,
            JavaFilePlacement::Conformance,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaSourceFileKind::CompilationUnit,
            source("conformance-file"),
        )))
    }

    fn negative_file(&mut self) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let declaration = JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: identifier("InvalidTypes"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![
                JavaMember::Constructor(private_constructor("InvalidTypes")),
                JavaMember::CompileFailField(JavaCompileFailField {
                    modifiers: vec![JavaModifier::Final],
                    expected_type: JavaType::primitive(JavaPrimitive::Int),
                    name: identifier("invalid"),
                    initializer: string_literal("missing"),
                }),
            ],
        };
        Ok(self.builder.file(TargetFile::new(
            path("src/test/java/org/polyrust/generated/InvalidTypes.java"),
            SourceRole::NegativeTest,
            JavaPackage::Generated,
            JavaFilePlacement::NegativeTest,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaSourceFileKind::CompilationUnit,
            source("negative-file"),
        )))
    }

    fn record_declaration(&self, id: CoreRecordId) -> Result<JavaTypeDeclaration, Vec<Diagnostic>> {
        let record = self.core.record(id).expect("verified record");
        let mut members = vec![
            JavaMember::Constructor(self.generated_record_constructor(
                self.records[&id],
                &record.header.name,
                record.header.visibility,
                &record.fields,
            )?),
            JavaMember::Method(self.value_equality_method(
                self.records[&id],
                &record.fields,
                JavaRuntimeCallable::SemanticEqual,
                JavaRuntimeMember::SemanticEquals,
            )?),
            JavaMember::Method(self.value_equality_method(
                self.records[&id],
                &record.fields,
                JavaRuntimeCallable::DeepEqual,
                JavaRuntimeMember::DeepEquals,
            )?),
        ];
        let mut interfaces = vec![JavaType::known(JavaKnownType::RuntimeSemanticValue)];
        for declaration in &self.core.module().declarations {
            if let CoreDeclaration::Implementation(implementation_id) = *declaration {
                let implementation = self
                    .core
                    .implementation(implementation_id)
                    .expect("verified implementation");
                if implementation.record == id {
                    interfaces.push(JavaType::Reference(JavaTypeName::Generated(
                        self.interfaces[&implementation.interface],
                    )));
                    for method in &implementation.methods {
                        members.push(JavaMember::Method(self.implementation_method(*method)?));
                    }
                }
            }
        }
        match self
            .features
            .mapping_for::<portable_build::Records>()
            .lower(
                &mut (),
                JavaRecordsInput::Declaration(Box::new(JavaRecordDeclarationInput {
                    declared: self.records[&id],
                    visibility: record.header.visibility,
                    name: record.header.name.clone(),
                    components: record
                        .fields
                        .iter()
                        .map(|field| {
                            let value = self.core.field(*field).expect("verified field");
                            Ok(JavaRecordComponent {
                                origin: JavaRecordComponentOrigin::Core(*field),
                                ty: self.ty(value.ty)?,
                                name: identifier(&value.header.name),
                            })
                        })
                        .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
                    heritage: if interfaces.is_empty() {
                        JavaHeritage::None
                    } else {
                        JavaHeritage::Interfaces(interfaces)
                    },
                    members,
                })),
            )? {
            JavaRecordsNode::Declaration(declaration) => Ok(declaration),
            JavaRecordsNode::Expression(_) => Err(vec![diagnostic(
                "Java Records mapping returned an expression for a declaration",
            )]),
        }
    }

    fn enum_declarations(
        &self,
        id: CoreEnumId,
    ) -> Result<Vec<JavaTypeDeclaration>, Vec<Diagnostic>> {
        let enumeration = self.core.enumeration(id).expect("verified enum");
        if self.enum_is_payload_free(id) {
            return match self.features.mapping_for::<portable_build::Enums>().lower(
                &mut (),
                JavaEnumsInput::Declaration {
                    declared: self.enums[&id],
                    visibility: enumeration.header.visibility,
                    name: enumeration.header.name.clone(),
                    variants: enumeration
                        .variants
                        .iter()
                        .map(|variant_id| {
                            let variant = self
                                .core
                                .variant(*variant_id)
                                .expect("verified enum variant");
                            JavaEnumVariantInput {
                                declared: self.enum_values[variant_id],
                                name: variant.header.name.clone(),
                            }
                        })
                        .collect(),
                },
            )? {
                JavaEnumsNode::Declaration(declaration) => Ok(vec![*declaration]),
                JavaEnumsNode::Expression(_) | JavaEnumsNode::Statement(_) => {
                    Err(vec![diagnostic(
                        "Java Enums mapping returned a value for a declaration",
                    )])
                }
            };
        }
        let visibility = java_visibility(enumeration.header.visibility);
        let enum_type = JavaType::Reference(JavaTypeName::Generated(self.enums[&id]));
        let mut output = vec![JavaTypeDeclaration {
            declared: Some(self.enums[&id]),
            kind: JavaDeclarationKind::SealedInterface,
            visibility,
            modifiers: vec![JavaModifier::Static],
            name: identifier(&enumeration.header.name),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: enumeration
                .variants
                .iter()
                .map(|variant| JavaType::Reference(JavaTypeName::Generated(self.variants[variant])))
                .collect(),
            members: vec![],
        }];
        for variant_id in &enumeration.variants {
            let variant = self.core.variant(*variant_id).expect("verified variant");
            let variant_type = self.variants[variant_id];
            output.push(JavaTypeDeclaration {
                declared: Some(variant_type),
                kind: JavaDeclarationKind::Record,
                visibility,
                modifiers: vec![JavaModifier::Static],
                name: identifier(&format!(
                    "{}{}",
                    enumeration.header.name, variant.header.name
                )),
                type_parameters: vec![],
                record_components: variant
                    .fields
                    .iter()
                    .map(|field| {
                        let value = self.core.field(*field).expect("verified field");
                        Ok(JavaRecordComponent {
                            origin: JavaRecordComponentOrigin::Core(*field),
                            ty: self.ty(value.ty)?,
                            name: identifier(&value.header.name),
                        })
                    })
                    .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
                heritage: JavaHeritage::Interfaces(vec![
                    enum_type.clone(),
                    JavaType::known(JavaKnownType::RuntimeSemanticValue),
                ]),
                permits: vec![],
                members: vec![
                    JavaMember::Constructor(self.generated_record_constructor(
                        variant_type,
                        &format!("{}{}", enumeration.header.name, variant.header.name),
                        enumeration.header.visibility,
                        &variant.fields,
                    )?),
                    JavaMember::Method(self.value_equality_method(
                        variant_type,
                        &variant.fields,
                        JavaRuntimeCallable::SemanticEqual,
                        JavaRuntimeMember::SemanticEquals,
                    )?),
                    JavaMember::Method(self.value_equality_method(
                        variant_type,
                        &variant.fields,
                        JavaRuntimeCallable::DeepEqual,
                        JavaRuntimeMember::DeepEquals,
                    )?),
                ],
            });
        }
        Ok(output)
    }

    fn enum_is_payload_free(&self, id: CoreEnumId) -> bool {
        self.core.enumeration(id).is_some_and(|enumeration| {
            !enumeration.variants.is_empty()
                && enumeration.variants.iter().all(|variant| {
                    self.core
                        .variant(*variant)
                        .is_some_and(|variant| variant.fields.is_empty())
                })
        })
    }

    fn enum_variant_expr(
        &self,
        enumeration: CoreEnumId,
        variant: CoreVariantId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        match self.features.mapping_for::<portable_build::Enums>().lower(
            &mut (),
            JavaEnumsInput::Variant {
                enumeration: self.enums[&enumeration],
                variant: self.enum_values[&variant],
            },
        )? {
            JavaEnumsNode::Expression(value) => Ok(*value),
            JavaEnumsNode::Declaration(_) | JavaEnumsNode::Statement(_) => Err(vec![diagnostic(
                "Java Enums mapping returned a declaration for a variant value",
            )]),
        }
    }

    fn generated_record_constructor(
        &self,
        owner: GeneratedTypeId,
        name: &str,
        visibility: Visibility,
        fields: &[CoreFieldId],
    ) -> Result<JavaConstructor, Vec<Diagnostic>> {
        let owner_type = JavaType::Reference(JavaTypeName::Generated(owner));
        let mut parameters = Vec::new();
        let mut statements = Vec::new();
        for field_id in fields {
            let field = self.core.field(*field_id).expect("verified field");
            let ty = self.ty(field.ty)?;
            let field_name = identifier(&field.header.name);
            let input = JavaExpr::local(ty.clone(), field_name.clone());
            let normalized = self.normalize_boundary_value(field.ty, input)?;
            statements.extend(normalized.statements);
            parameters.push(JavaParameter {
                ty: ty.clone(),
                name: field_name.clone(),
                final_parameter: true,
            });
            statements.push(JavaStmt::Assign {
                target: JavaExpr {
                    ty: ty.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Field {
                        receiver: Box::new(JavaExpr {
                            ty: owner_type.clone(),
                            precedence: JavaPrecedence::Primary,
                            kind: JavaExprKind::Value(JavaValueRef::This),
                        }),
                        field: JavaFieldRef::Generated {
                            owner,
                            field: *field_id,
                            name: field_name,
                            ty,
                        },
                    },
                },
                value: normalized.value,
            });
        }
        Ok(JavaConstructor {
            modifiers: vec![visibility_modifier(visibility)],
            name: identifier(name),
            parameters,
            body: JavaBlock::new(statements),
        })
    }

    fn value_equality_method(
        &self,
        owner: GeneratedTypeId,
        fields: &[CoreFieldId],
        callable: JavaRuntimeCallable,
        member: JavaRuntimeMember,
    ) -> Result<JavaMethod, Vec<Diagnostic>> {
        let object = JavaType::known(JavaKnownType::Object);
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let owner_type = JavaType::Reference(JavaTypeName::Generated(owner));
        let this = JavaExpr {
            ty: owner_type.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::This),
        };
        let other = JavaExpr::local(owner_type.clone(), identifier("otherValue"));
        let mut equal = bool_literal(true);
        for field in fields {
            let metadata = self.core.field(*field).expect("verified field");
            let field_type = self.ty(metadata.ty)?;
            equal = binary(
                JavaBinaryOperator::LogicalAnd,
                equal,
                runtime_call(
                    callable,
                    vec![
                        member_call(
                            this.clone(),
                            &metadata.header.name,
                            vec![],
                            field_type.clone(),
                            JavaMemberOrigin::GeneratedField(*field),
                        ),
                        member_call(
                            other.clone(),
                            &metadata.header.name,
                            vec![],
                            field_type,
                            JavaMemberOrigin::GeneratedField(*field),
                        ),
                    ],
                    boolean.clone(),
                ),
                boolean.clone(),
            );
        }
        Ok(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![JavaAnnotation::Override],
            modifiers: vec![JavaModifier::Public],
            type_parameters: vec![],
            return_type: boolean.clone(),
            name: identifier(member.name()),
            parameters: vec![JavaParameter {
                ty: object.clone(),
                name: identifier("other"),
                final_parameter: true,
            }],
            body: Some(JavaBlock::new(vec![
                JavaStmt::If {
                    condition: unary(
                        JavaUnaryOperator::Not,
                        instance_of(
                            JavaExpr::local(object, identifier("other")),
                            owner_type,
                            Some(identifier("otherValue")),
                        ),
                        boolean.clone(),
                    ),
                    then_block: JavaBlock::new(vec![JavaStmt::Return(Some(bool_literal(false)))]),
                    else_block: None,
                },
                JavaStmt::Return(Some(equal)),
            ])),
        })
    }

    fn interface_declaration(
        &self,
        id: CoreInterfaceId,
    ) -> Result<JavaTypeDeclaration, Vec<Diagnostic>> {
        let interface = self.core.interface(id).expect("verified interface");
        let permits = self
            .core
            .implementations()
            .iter()
            .filter(|implementation| implementation.interface == id)
            .map(|implementation| {
                JavaType::Reference(JavaTypeName::Generated(
                    self.records[&implementation.record],
                ))
            })
            .collect();
        let methods = interface
            .methods
            .iter()
            .map(|method_id| {
                let method = self
                    .core
                    .interface_method(*method_id)
                    .expect("verified method");
                Ok(JavaInterfaceMethodInput {
                    declared: self.interface_methods[method_id],
                    name: method.header.name.clone(),
                    parameters: self.parameters(&method.parameters)?,
                    return_type: self.poly_result_type(method.return_type)?,
                })
            })
            .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
        match self
            .features
            .mapping_for::<portable_build::Interfaces>()
            .lower(
                &mut (),
                JavaInterfacesInput::Declaration(Box::new(JavaInterfaceDeclarationInput {
                    declared: self.interfaces[&id],
                    visibility: interface.header.visibility,
                    name: interface.header.name.clone(),
                    permits,
                    methods,
                })),
            )? {
            JavaInterfacesNode::Declaration(declaration) => Ok(*declaration),
            JavaInterfacesNode::Method(_) | JavaInterfacesNode::Expression(_) => {
                Err(vec![diagnostic(
                    "Java Interfaces mapping returned the wrong declaration node",
                )])
            }
        }
    }

    fn parameters(
        &self,
        values: &[portable_core_ir::CoreParameter],
    ) -> Result<Vec<JavaParameter>, Vec<Diagnostic>> {
        values
            .iter()
            .map(|parameter| {
                Ok(JavaParameter {
                    ty: self.ty(parameter.ty)?,
                    name: identifier(&parameter.header.name),
                    final_parameter: true,
                })
            })
            .collect()
    }

    fn constant_field(&self, id: CoreConstantId) -> Result<JavaField, Vec<Diagnostic>> {
        let value = self.core.constant(id).expect("verified constant");
        match self
            .features
            .mapping_for::<portable_build::Constants>()
            .lower(
                &mut (),
                JavaConstantsInput::Declaration {
                    declared: self.constants[&id],
                    visibility: value.header.visibility,
                    name: value.header.name.clone(),
                    ty: self.ty(value.ty)?,
                    initializer: Box::new(self.constant_expr(&value.value, value.ty)?),
                },
            )? {
            JavaConstantsNode::Declaration(field) => Ok(field),
            JavaConstantsNode::Expression(_) => Err(vec![diagnostic(
                "Java Constants mapping returned an expression for a declaration",
            )]),
        }
    }

    fn function_method(&self, id: CoreFunctionId) -> Result<JavaMethod, Vec<Diagnostic>> {
        let value = self.core.function(id).expect("verified function");
        let (parameters, mut boundary) = self.callable_parameters(&value.parameters)?;
        let mut body = self.block(value.body, BlockMode::ReturnResult, value.return_type)?;
        boundary.append(&mut body.statements);
        match self
            .features
            .mapping_for::<portable_build::Functions>()
            .lower(
                &mut (),
                JavaFunctionsInput::Declaration(Box::new(JavaFunctionDeclarationInput {
                    declared: JavaMethodDeclaration::Callable(self.functions[&id]),
                    visibility: value.header.visibility,
                    name: value.header.name.clone(),
                    parameters,
                    return_type: self.poly_result_type(value.return_type)?,
                    body: JavaBlock::new(boundary),
                })),
            )? {
            JavaFunctionsNode::Declaration(method) => Ok(method),
            JavaFunctionsNode::Expression(_) => Err(vec![diagnostic(
                "Java Functions mapping returned an expression for a declaration",
            )]),
        }
    }

    fn public_tagged_value_factories(&self) -> Result<Vec<JavaMember>, Vec<Diagnostic>> {
        let mut members = Vec::new();
        for (id, kind) in self.core.types().iter() {
            match kind {
                CoreType::Option(inner) => {
                    let suffix = self.factory_type_suffix(id)?;
                    let option_type = self.ty(id)?;
                    members.push(public_factory_method(
                        &format!("__polyrust_noneOf{suffix}"),
                        option_type.clone(),
                        vec![],
                        ExprPlan::pure(runtime_call(
                            JavaRuntimeCallable::OptionNone,
                            vec![],
                            option_type.clone(),
                        )),
                    ));
                    let payload_type = self.ty(*inner)?;
                    let input = JavaExpr::local(payload_type.clone(), identifier("value"));
                    let normalized = self.normalize_boundary_value(*inner, input)?;
                    members.push(public_factory_method(
                        &format!("__polyrust_someOf{suffix}"),
                        option_type.clone(),
                        vec![JavaParameter {
                            ty: payload_type,
                            name: identifier("value"),
                            final_parameter: true,
                        }],
                        ExprPlan {
                            statements: normalized.statements,
                            value: runtime_call(
                                JavaRuntimeCallable::OptionSome,
                                vec![normalized.value],
                                option_type,
                            ),
                        },
                    ));
                }
                CoreType::Result { ok, error } => {
                    let suffix = self.factory_type_suffix(id)?;
                    let result_type = self.ty(id)?;
                    for (label, payload, callable) in [
                        ("ok", *ok, JavaRuntimeCallable::ValueResultOk),
                        ("error", *error, JavaRuntimeCallable::ValueResultErr),
                    ] {
                        let payload_type = self.ty(payload)?;
                        let input = JavaExpr::local(payload_type.clone(), identifier("value"));
                        let normalized = self.normalize_boundary_value(payload, input)?;
                        members.push(public_factory_method(
                            &format!("__polyrust_{label}Of{suffix}"),
                            result_type.clone(),
                            vec![JavaParameter {
                                ty: payload_type,
                                name: identifier("value"),
                                final_parameter: true,
                            }],
                            ExprPlan {
                                statements: normalized.statements,
                                value: runtime_call(
                                    callable,
                                    vec![normalized.value],
                                    result_type.clone(),
                                ),
                            },
                        ));
                    }
                }
                CoreType::Unit
                | CoreType::Bool
                | CoreType::I32
                | CoreType::I64
                | CoreType::F64
                | CoreType::Char
                | CoreType::String
                | CoreType::Bytes
                | CoreType::List(_)
                | CoreType::Record(_)
                | CoreType::Enum(_)
                | CoreType::Interface(_) => {}
            }
        }
        Ok(members)
    }

    fn factory_type_suffix(&self, id: CoreTypeId) -> Result<String, Vec<Diagnostic>> {
        let Some(kind) = self.core.types().get(id) else {
            return Err(vec![diagnostic(
                "missing CoreIR type for Java value factory",
            )]);
        };
        let suffix = match kind {
            CoreType::Unit => "Unit".to_owned(),
            CoreType::Bool => "Bool".to_owned(),
            CoreType::I32 => "I32".to_owned(),
            CoreType::I64 => "I64".to_owned(),
            CoreType::F64 => "F64".to_owned(),
            CoreType::Char => "Char".to_owned(),
            CoreType::String => "String".to_owned(),
            CoreType::Bytes => "Bytes".to_owned(),
            CoreType::List(element) => {
                length_prefixed_type("List", &self.factory_type_suffix(*element)?)
            }
            CoreType::Option(inner) => {
                length_prefixed_type("Option", &self.factory_type_suffix(*inner)?)
            }
            CoreType::Result { ok, error } => {
                let ok = self.factory_type_suffix(*ok)?;
                let error = self.factory_type_suffix(*error)?;
                format!("Result{}_{}{}_{}", ok.len(), ok, error.len(), error)
            }
            CoreType::Record(record) => {
                let name = identifier(
                    &self
                        .core
                        .record(*record)
                        .expect("verified record")
                        .header
                        .name,
                );
                length_prefixed_type("Record", name.as_str())
            }
            CoreType::Enum(enumeration) => {
                let name = identifier(
                    &self
                        .core
                        .enumeration(*enumeration)
                        .expect("verified enum")
                        .header
                        .name,
                );
                length_prefixed_type("Enum", name.as_str())
            }
            CoreType::Interface(interface) => {
                let name = identifier(
                    &self
                        .core
                        .interface(*interface)
                        .expect("verified interface")
                        .header
                        .name,
                );
                length_prefixed_type("Interface", name.as_str())
            }
        };
        Ok(suffix)
    }

    fn implementation_method(
        &self,
        id: CoreImplementationMethodId,
    ) -> Result<JavaMethod, Vec<Diagnostic>> {
        let value = self
            .core
            .implementation_method(id)
            .expect("verified implementation method");
        let interface_method = self
            .core
            .interface_method(value.interface_method)
            .expect("verified interface method");
        let (parameters, mut boundary) = self.callable_parameters(&value.parameters)?;
        let mut body = self.block(value.body, BlockMode::ReturnResult, value.return_type)?;
        boundary.append(&mut body.statements);
        match self
            .features
            .mapping_for::<portable_build::Interfaces>()
            .lower(
                &mut (),
                JavaInterfacesInput::Implementation(Box::new(JavaInterfaceImplementationInput {
                    method: id,
                    interface_method: self.interface_methods[&value.interface_method],
                    interface_method_name: interface_method.header.name.clone(),
                    parameters,
                    return_type: self.poly_result_type(value.return_type)?,
                    body: JavaBlock::new(boundary),
                })),
            )? {
            JavaInterfacesNode::Method(method) => Ok(*method),
            JavaInterfacesNode::Declaration(_) | JavaInterfacesNode::Expression(_) => {
                Err(vec![diagnostic(
                    "Java Interfaces mapping returned the wrong implementation node",
                )])
            }
        }
    }

    fn callable_parameters(
        &self,
        values: &[portable_core_ir::CoreParameter],
    ) -> Result<(Vec<JavaParameter>, Vec<JavaStmt>), Vec<Diagnostic>> {
        let mut parameters = Vec::with_capacity(values.len());
        let mut boundary = Vec::new();
        for (index, parameter) in values.iter().enumerate() {
            let ty = self.ty(parameter.ty)?;
            if matches!(ty, JavaType::Primitive(_)) {
                parameters.push(JavaParameter {
                    ty,
                    name: identifier(&parameter.header.name),
                    final_parameter: true,
                });
                continue;
            }
            let input_name = JavaIdentifier::new(format!("__polyrust_input_{index}"))
                .expect("internal Java parameter identifier is valid");
            let input = JavaExpr::local(ty.clone(), input_name.clone());
            let normalized = self.normalize_boundary_value(parameter.ty, input)?;
            parameters.push(JavaParameter {
                ty: ty.clone(),
                name: input_name,
                final_parameter: true,
            });
            boundary.extend(normalized.statements);
            boundary.push(JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty,
                name: identifier(&parameter.header.name),
                value: Some(normalized.value),
            });
        }
        Ok((parameters, boundary))
    }

    fn normalize_boundary_value(
        &self,
        core_type: CoreTypeId,
        input: JavaExpr,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let Some(kind) = self.core.types().get(core_type) else {
            return Err(vec![diagnostic("missing boundary CoreIR type")]);
        };
        match kind {
            CoreType::Unit
            | CoreType::Char
            | CoreType::Bytes
            | CoreType::Record(_)
            | CoreType::Enum(_)
            | CoreType::Interface(_) => Ok(ExprPlan::pure(known_generic_call(
                JavaKnownCallable::ObjectsRequireNonNull,
                vec![input.clone()],
                input.ty,
            ))),
            CoreType::String => Ok(ExprPlan::pure(runtime_call(
                JavaRuntimeCallable::RequireScalarString,
                vec![input.clone()],
                input.ty,
            ))),
            CoreType::Bool | CoreType::I32 | CoreType::I64 | CoreType::F64 => {
                Ok(ExprPlan::pure(input))
            }
            CoreType::List(element) => self.normalize_boundary_list(*element, input),
            CoreType::Option(inner) => self.normalize_boundary_option(*inner, input),
            CoreType::Result { ok, error } => self.normalize_boundary_result(*ok, *error, input),
        }
    }

    fn normalize_boundary_list(
        &self,
        element: CoreTypeId,
        input: JavaExpr,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let list_type = input.ty.clone();
        let element_type = self.ty(element)?.boxed();
        let mutable_type = JavaType::Generic {
            raw: JavaTypeName::Known(JavaKnownType::ArrayList),
            arguments: vec![element_type.clone()],
        };
        let (output_name, output) = self.temporary("boundaryList", mutable_type.clone());
        let (item_name, item) = self.temporary("boundaryItem", element_type.clone());
        let normalized = self.normalize_boundary_value(element, item)?;
        let mut loop_body = normalized.statements;
        loop_body.push(JavaStmt::Expression(member_call(
            output.clone(),
            JavaKnownMethod::ArrayListAdd.name().text(),
            vec![normalized.value],
            JavaType::primitive(JavaPrimitive::Boolean),
            JavaMemberOrigin::Known(JavaKnownMethod::ArrayListAdd),
        )));
        Ok(ExprPlan {
            statements: vec![
                JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: mutable_type.clone(),
                    name: output_name,
                    value: Some(new_known(
                        JavaKnownConstructor::ArrayList,
                        mutable_type,
                        vec![],
                    )),
                },
                JavaStmt::ForEach {
                    binding_type: element_type,
                    binding: item_name,
                    iterable: input,
                    body: JavaBlock::new(loop_body),
                },
            ],
            value: known_generic_call(JavaKnownCallable::ListCopyOf, vec![output], list_type),
        })
    }

    fn normalize_boundary_option(
        &self,
        inner: CoreTypeId,
        input: JavaExpr,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let option_type = input.ty.clone();
        let (result_name, result) = self.temporary("boundaryOption", option_type.clone());
        let payload = runtime_call(
            JavaRuntimeCallable::OptionValue,
            vec![input.clone()],
            self.ty(inner)?.boxed(),
        );
        let normalized = self.normalize_boundary_value(inner, payload)?;
        let mut some_statements = normalized.statements;
        some_statements.push(JavaStmt::Assign {
            target: result.clone(),
            value: runtime_call(
                JavaRuntimeCallable::OptionSome,
                vec![normalized.value],
                option_type.clone(),
            ),
        });
        Ok(ExprPlan {
            statements: vec![
                JavaStmt::Local {
                    finality: JavaLocalFinality::Mutable,
                    ty: option_type.clone(),
                    name: result_name,
                    value: None,
                },
                JavaStmt::If {
                    condition: runtime_call(
                        JavaRuntimeCallable::OptionIsSome,
                        vec![input],
                        JavaType::primitive(JavaPrimitive::Boolean),
                    ),
                    then_block: JavaBlock::new(some_statements),
                    else_block: Some(JavaBlock::new(vec![JavaStmt::Assign {
                        target: result.clone(),
                        value: runtime_call(JavaRuntimeCallable::OptionNone, vec![], option_type),
                    }])),
                },
            ],
            value: result,
        })
    }

    fn normalize_boundary_result(
        &self,
        ok: CoreTypeId,
        error: CoreTypeId,
        input: JavaExpr,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let result_type = input.ty.clone();
        let (result_name, result) = self.temporary("boundaryResult", result_type.clone());
        let ok_value = runtime_call(
            JavaRuntimeCallable::ValueResultValue,
            vec![input.clone()],
            self.ty(ok)?.boxed(),
        );
        let normalized_ok = self.normalize_boundary_value(ok, ok_value)?;
        let mut ok_statements = normalized_ok.statements;
        ok_statements.push(JavaStmt::Assign {
            target: result.clone(),
            value: runtime_call(
                JavaRuntimeCallable::ValueResultOk,
                vec![normalized_ok.value],
                result_type.clone(),
            ),
        });
        let error_value = runtime_call(
            JavaRuntimeCallable::ValueResultError,
            vec![input.clone()],
            self.ty(error)?.boxed(),
        );
        let normalized_error = self.normalize_boundary_value(error, error_value)?;
        let mut error_statements = normalized_error.statements;
        error_statements.push(JavaStmt::Assign {
            target: result.clone(),
            value: runtime_call(
                JavaRuntimeCallable::ValueResultErr,
                vec![normalized_error.value],
                result_type.clone(),
            ),
        });
        Ok(ExprPlan {
            statements: vec![
                JavaStmt::Local {
                    finality: JavaLocalFinality::Mutable,
                    ty: result_type,
                    name: result_name,
                    value: None,
                },
                JavaStmt::If {
                    condition: runtime_call(
                        JavaRuntimeCallable::ValueResultIsOk,
                        vec![input],
                        JavaType::primitive(JavaPrimitive::Boolean),
                    ),
                    then_block: JavaBlock::new(ok_statements),
                    else_block: Some(JavaBlock::new(error_statements)),
                },
            ],
            value: result,
        })
    }

    fn native_test_file(&mut self) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let declaration = self.test_declaration("GeneratedTest")?;
        Ok(self.builder.file(TargetFile::new(
            path("src/test/java/org/polyrust/generated/GeneratedTest.java"),
            SourceRole::NativeTest,
            JavaPackage::Generated,
            JavaFilePlacement::NativeTest,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaSourceFileKind::CompilationUnit,
            source("native-test-file"),
        )))
    }

    fn block(
        &self,
        id: CoreBlockId,
        mode: BlockMode,
        callable_return: CoreTypeId,
    ) -> Result<JavaBlock, Vec<Diagnostic>> {
        let block = self
            .core
            .blocks()
            .get(id)
            .ok_or_else(|| vec![diagnostic("missing CoreIR block")])?;
        let mut statements = Vec::new();
        for statement in &block.statements {
            match statement {
                CoreStatement::Let { local, value, .. } => {
                    let binding = self.core.local(*local).expect("verified local");
                    let plan = self.expr_plan(*value, callable_return)?;
                    statements.extend(plan.statements);
                    statements.push(
                        self.features
                            .mapping_for::<portable_build::LocalBindings>()
                            .lower(
                                &mut (),
                                JavaLocalBindingInput {
                                    name: binding.name.clone(),
                                    ty: self.ty(binding.ty)?,
                                    value: plan.value,
                                },
                            )?,
                    );
                }
                CoreStatement::ForEach {
                    binding,
                    iterable,
                    body,
                    ..
                } => {
                    let binding = self.core.local(*binding).expect("verified local");
                    let iterable = self.expr_plan(*iterable, callable_return)?;
                    statements.extend(iterable.statements);
                    statements.push(self.features.mapping_for::<portable_build::Loops>().lower(
                        &mut (),
                        JavaLoopsInput::ForEach {
                            binding_type: self.ty(binding.ty)?,
                            binding: binding.name.clone(),
                            iterable: iterable.value,
                            body: self.block(*body, BlockMode::StatementBody, callable_return)?,
                        },
                    )?);
                }
                CoreStatement::Return { value, .. } => {
                    let plan = match value {
                        Some(value) => self.expr_plan(*value, callable_return)?,
                        None => ExprPlan::pure(self.lower_unit_value()?),
                    };
                    statements.extend(plan.statements);
                    statements.push(JavaStmt::Return(Some(
                        self.success_result(plan.value, callable_return)?,
                    )));
                }
                CoreStatement::Evaluate { value, .. } => {
                    let plan = self.expr_plan(*value, callable_return)?;
                    self.append_evaluation(&mut statements, plan);
                }
            }
        }
        if let Some(result) = block.result {
            let plan = self.expr_plan(result, callable_return)?;
            match mode {
                BlockMode::ReturnResult => {
                    statements.extend(plan.statements);
                    statements.push(JavaStmt::Return(Some(
                        self.success_result(plan.value, callable_return)?,
                    )));
                }
                BlockMode::AssignResult { ref target } => {
                    statements.extend(plan.statements);
                    statements.push(JavaStmt::Assign {
                        target: target.as_ref().clone(),
                        value: plan.value,
                    });
                }
                BlockMode::StatementBody => self.append_evaluation(&mut statements, plan),
            }
        } else {
            match mode {
                BlockMode::ReturnResult => statements.push(JavaStmt::Return(Some(
                    self.success_result(self.lower_unit_value()?, callable_return)?,
                ))),
                BlockMode::AssignResult { target } => statements.push(JavaStmt::Assign {
                    target: *target,
                    value: self.lower_unit_value()?,
                }),
                BlockMode::StatementBody => {}
            }
        }
        Ok(JavaBlock::new(statements))
    }

    fn append_evaluation(&self, statements: &mut Vec<JavaStmt>, mut plan: ExprPlan) {
        statements.append(&mut plan.statements);
        let ty = plan.value.ty.clone();
        let (name, _) = self.temporary("evaluate", ty.clone());
        statements.push(JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty,
            name,
            value: Some(plan.value),
        });
    }

    fn lower_function_expr(&self, input: JavaFunctionsInput) -> Result<JavaExpr, Vec<Diagnostic>> {
        match self
            .features
            .mapping_for::<portable_build::Functions>()
            .lower(&mut (), input)?
        {
            JavaFunctionsNode::Expression(value) => Ok(value),
            JavaFunctionsNode::Declaration(_) => Err(vec![diagnostic(
                "Java Functions mapping returned a declaration for an expression",
            )]),
        }
    }

    fn lower_record_expr(&self, input: JavaRecordsInput) -> Result<JavaExpr, Vec<Diagnostic>> {
        match self
            .features
            .mapping_for::<portable_build::Records>()
            .lower(&mut (), input)?
        {
            JavaRecordsNode::Expression(value) => Ok(value),
            JavaRecordsNode::Declaration(_) => Err(vec![diagnostic(
                "Java Records mapping returned a declaration for an expression",
            )]),
        }
    }

    fn lower_interface_expr(
        &self,
        input: JavaInterfacesInput,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        match self
            .features
            .mapping_for::<portable_build::Interfaces>()
            .lower(&mut (), input)?
        {
            JavaInterfacesNode::Expression(value) => Ok(*value),
            JavaInterfacesNode::Declaration(_) | JavaInterfacesNode::Method(_) => {
                Err(vec![diagnostic(
                    "Java Interfaces mapping returned a declaration for an expression",
                )])
            }
        }
    }

    fn expr_plan(
        &self,
        id: CoreExprId,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let expression = self
            .core
            .expressions()
            .get(id)
            .ok_or_else(|| vec![diagnostic("missing CoreIR expression")])?;
        let ty = self.ty(expression.ty)?;
        match &expression.kind {
            CoreExprKind::Literal(value) => Ok(ExprPlan::pure(self.value(value, expression.ty)?)),
            CoreExprKind::Local(id) => {
                let local_value = self.core.local(*id).expect("verified local");
                Ok(ExprPlan::pure(self.lower_function_expr(
                    JavaFunctionsInput::Local {
                        ty,
                        name: local_value.name.clone(),
                    },
                )?))
            }
            CoreExprKind::Constant(id) => Ok(ExprPlan::pure(JavaExpr {
                ty,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(
                    self.constants[id],
                ))),
            })),
            CoreExprKind::SelfValue(id) => Ok(ExprPlan::pure(JavaExpr {
                ty: JavaType::Reference(JavaTypeName::Generated(self.records[id])),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::This),
            })),
            CoreExprKind::ConstructRecord { record, fields } => {
                self.construct_generated_plan(self.records[record], fields, ty, callable_return)
            }
            CoreExprKind::ConstructEnum {
                enumeration,
                variant,
                fields,
            } if self.enum_is_payload_free(*enumeration) => {
                if !fields.is_empty() {
                    return Err(vec![diagnostic(
                        "payload-free Java enum construction cannot contain fields",
                    )]);
                }
                Ok(ExprPlan::pure(
                    self.enum_variant_expr(*enumeration, *variant)?,
                ))
            }
            CoreExprKind::ConstructEnum {
                variant, fields, ..
            } => self.construct_generated_plan(self.variants[variant], fields, ty, callable_return),
            CoreExprKind::ConstructSome(value) => {
                let mut plan = self.expr_plan(*value, callable_return)?;
                plan.value = self
                    .features
                    .mapping_for::<portable_build::OptionValues>()
                    .lower(
                        &mut (),
                        JavaOptionInput::Some {
                            value: Box::new(plan.value),
                            result: ty,
                        },
                    )?;
                Ok(plan)
            }
            CoreExprKind::ConstructNone { .. } => {
                let value = self
                    .features
                    .mapping_for::<portable_build::OptionValues>()
                    .lower(&mut (), JavaOptionInput::None { result: ty })?;
                Ok(ExprPlan::pure(value))
            }
            CoreExprKind::ConstructOk { value, .. } => {
                let mut plan = self.expr_plan(*value, callable_return)?;
                plan.value = self
                    .features
                    .mapping_for::<portable_build::ResultValues>()
                    .lower(
                        &mut (),
                        JavaResultInput::Ok {
                            value: plan.value,
                            result: ty,
                        },
                    )?;
                Ok(plan)
            }
            CoreExprKind::ConstructErr { value, .. } => {
                let mut plan = self.expr_plan(*value, callable_return)?;
                plan.value = self
                    .features
                    .mapping_for::<portable_build::ResultValues>()
                    .lower(
                        &mut (),
                        JavaResultInput::Err {
                            value: plan.value,
                            result: ty,
                        },
                    )?;
                Ok(plan)
            }
            CoreExprKind::ConstructList { elements, .. } => {
                let (statements, values) = self.expr_list(elements, callable_return)?;
                let value = self
                    .features
                    .mapping_for::<portable_build::ListValues>()
                    .lower(
                        &mut (),
                        JavaListInput {
                            elements: values,
                            result: ty,
                        },
                    )?;
                Ok(ExprPlan { statements, value })
            }
            CoreExprKind::CoerceInterface { value, .. } => {
                let mut plan = self.expr_plan(*value, callable_return)?;
                plan.value = self.lower_interface_expr(JavaInterfacesInput::Coerce {
                    value: Box::new(plan.value),
                    result: ty,
                })?;
                Ok(plan)
            }
            CoreExprKind::Field { value, field } => {
                let field_value = self.core.field(*field).expect("verified field");
                let mut plan = self.expr_plan(*value, callable_return)?;
                plan.value = self.lower_record_expr(JavaRecordsInput::Field {
                    receiver: Box::new(plan.value),
                    name: field_value.header.name.clone(),
                    result: ty,
                    origin: JavaMemberOrigin::GeneratedField(*field),
                })?;
                Ok(plan)
            }
            CoreExprKind::Call {
                function,
                arguments,
            } => {
                let function_value = self.core.function(*function).expect("verified function");
                let (statements, arguments) = self.expr_list(arguments, callable_return)?;
                let result = self.poly_result_type(function_value.return_type)?;
                let signature = JavaMethodSignature {
                    receiver: None,
                    parameters: function_value
                        .parameters
                        .iter()
                        .map(|parameter| self.ty(parameter.ty))
                        .collect::<Result<Vec<_>, _>>()?,
                    result: result.clone(),
                    checked_exceptions: vec![],
                    nullable_result: false,
                    pure: true,
                };
                let call = self.lower_function_expr(JavaFunctionsInput::Call {
                    result,
                    callable: Box::new(JavaCallableRef::Generated {
                        symbol: self.functions[function],
                        signature,
                    }),
                    arguments,
                })?;
                self.propagate_call(statements, call, ty, callable_return)
            }
            CoreExprKind::StaticMethodCall {
                method,
                receiver,
                arguments,
                ..
            } => {
                let method_value = self
                    .core
                    .implementation_method(*method)
                    .expect("verified method");
                let interface_method = self
                    .core
                    .interface_method(method_value.interface_method)
                    .expect("verified interface method");
                let mut receiver =
                    self.stabilize_plan(self.expr_plan(*receiver, callable_return)?, "receiver");
                let (argument_statements, arguments) =
                    self.expr_list(arguments, callable_return)?;
                receiver.statements.extend(argument_statements);
                let result = self.poly_result_type(method_value.return_type)?;
                let call = self.lower_interface_expr(JavaInterfacesInput::ConcreteCall(
                    Box::new(JavaConcreteInterfaceCallInput {
                        receiver: receiver.value,
                        interface_method_name: interface_method.header.name.clone(),
                        arguments,
                        result,
                        method: *method,
                    }),
                ))?;
                self.propagate_call(receiver.statements, call, ty, callable_return)
            }
            CoreExprKind::InterfaceCall {
                method,
                receiver,
                arguments,
                ..
            } => {
                let method_value = self
                    .core
                    .interface_method(*method)
                    .expect("verified interface method");
                let mut receiver =
                    self.stabilize_plan(self.expr_plan(*receiver, callable_return)?, "receiver");
                let (argument_statements, arguments) =
                    self.expr_list(arguments, callable_return)?;
                receiver.statements.extend(argument_statements);
                let result = self.poly_result_type(method_value.return_type)?;
                let signature = JavaMethodSignature {
                    receiver: Some(JavaType::Reference(JavaTypeName::Generated(
                        self.interfaces[&method_value.interface],
                    ))),
                    parameters: method_value
                        .parameters
                        .iter()
                        .map(|parameter| self.ty(parameter.ty))
                        .collect::<Result<Vec<_>, _>>()?,
                    result: result.clone(),
                    checked_exceptions: vec![],
                    nullable_result: false,
                    pure: true,
                };
                let call = self.lower_interface_expr(JavaInterfacesInput::InterfaceCall(
                    Box::new(JavaInterfaceCallInput {
                        receiver: receiver.value,
                        arguments,
                        result,
                        symbol: self.interface_methods[method],
                        signature,
                    }),
                ))?;
                self.propagate_call(receiver.statements, call, ty, callable_return)
            }
            CoreExprKind::Intrinsic(value) => {
                self.intrinsic_plan(value, expression.ty, callable_return)
            }
            CoreExprKind::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition = self.expr_plan(*condition, callable_return)?;
                let (name, result_local) = self.temporary("ifResult", ty.clone());
                match self
                    .features
                    .mapping_for::<portable_build::Conditionals>()
                    .lower(
                        &mut (),
                        JavaConditionalsInput::Value(Box::new(JavaConditionalValueInput {
                            prefix: condition.statements,
                            condition: condition.value,
                            result_name: name,
                            result_type: ty,
                            then_block: self.block(
                                *then_block,
                                BlockMode::AssignResult {
                                    target: Box::new(result_local.clone()),
                                },
                                callable_return,
                            )?,
                            else_block: self.block(
                                *else_block,
                                BlockMode::AssignResult {
                                    target: Box::new(result_local),
                                },
                                callable_return,
                            )?,
                        })),
                    )? {
                    JavaConditionalsNode::Value { statements, value } => Ok(ExprPlan {
                        statements,
                        value: *value,
                    }),
                    JavaConditionalsNode::Statement(_) => Err(vec![diagnostic(
                        "Java Conditionals mapping returned a statement for a value",
                    )]),
                }
            }
            CoreExprKind::Match { value, arms } => {
                self.match_plan(*value, arms, expression.ty, callable_return)
            }
            CoreExprKind::Block(block) => {
                let (name, result_local) = self.temporary("blockResult", ty.clone());
                let mut statements = vec![JavaStmt::Local {
                    finality: JavaLocalFinality::Mutable,
                    ty,
                    name,
                    value: None,
                }];
                statements.extend(
                    self.block(
                        *block,
                        BlockMode::AssignResult {
                            target: Box::new(result_local.clone()),
                        },
                        callable_return,
                    )?
                    .statements,
                );
                Ok(ExprPlan {
                    statements,
                    value: result_local,
                })
            }
        }
    }

    fn construct_generated_plan(
        &self,
        owner: GeneratedTypeId,
        fields: &[CoreExprField],
        result_type: JavaType,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let ids = fields.iter().map(|field| field.value).collect::<Vec<_>>();
        let (statements, arguments) = self.expr_list(&ids, callable_return)?;
        let value = self.lower_record_expr(JavaRecordsInput::Construction {
            owner,
            arguments,
            result: result_type,
        })?;
        Ok(ExprPlan { statements, value })
    }

    fn lower_unit_value(&self) -> Result<JavaExpr, Vec<Diagnostic>> {
        self.features
            .mapping_for::<portable_build::UnitValues>()
            .lower(&mut (), ())
    }

    fn value(&self, value: &CoreValue, expected: CoreTypeId) -> Result<JavaExpr, Vec<Diagnostic>> {
        let ty = self.ty(expected)?;
        match value {
            CoreValue::Unit => self.lower_unit_value(),
            CoreValue::Bool(value) => self
                .features
                .mapping_for::<portable_build::BoolValues>()
                .lower(&mut (), *value),
            CoreValue::I32(value) => self
                .features
                .mapping_for::<portable_build::I32Values>()
                .lower(&mut (), *value),
            CoreValue::I64(value) => self
                .features
                .mapping_for::<portable_build::I64Values>()
                .lower(&mut (), *value),
            CoreValue::F64(value) => self
                .features
                .mapping_for::<portable_build::F64Values>()
                .lower(&mut (), value.0),
            CoreValue::Char(value) => self
                .features
                .mapping_for::<portable_build::CharValues>()
                .lower(&mut (), *value),
            CoreValue::String(value) => self
                .features
                .mapping_for::<portable_build::TextValues>()
                .lower(&mut (), value.clone()),
            CoreValue::Bytes(values) => self
                .features
                .mapping_for::<portable_build::BytesValues>()
                .lower(
                    &mut (),
                    JavaBytesInput {
                        values: values.clone(),
                        result: ty,
                    },
                ),
            CoreValue::List(values) => {
                let CoreType::List(element) =
                    self.core.types().get(expected).expect("verified list type")
                else {
                    return Err(vec![diagnostic("list value does not have a list type")]);
                };
                let elements = values
                    .iter()
                    .map(|value| self.value(value, *element))
                    .collect::<Result<Vec<_>, _>>()?;
                self.features
                    .mapping_for::<portable_build::ListValues>()
                    .lower(
                        &mut (),
                        JavaListInput {
                            elements,
                            result: ty,
                        },
                    )
            }
            CoreValue::None => self
                .features
                .mapping_for::<portable_build::OptionValues>()
                .lower(&mut (), JavaOptionInput::None { result: ty }),
            CoreValue::Some(value) => {
                let CoreType::Option(inner) = self
                    .core
                    .types()
                    .get(expected)
                    .expect("verified option type")
                else {
                    return Err(vec![diagnostic("some value does not have an option type")]);
                };
                self.features
                    .mapping_for::<portable_build::OptionValues>()
                    .lower(
                        &mut (),
                        JavaOptionInput::Some {
                            value: Box::new(self.value(value, *inner)?),
                            result: ty,
                        },
                    )
            }
            CoreValue::Ok(value) => {
                let CoreType::Result { ok, .. } = self
                    .core
                    .types()
                    .get(expected)
                    .expect("verified result type")
                else {
                    return Err(vec![diagnostic("ok value does not have a result type")]);
                };
                self.features
                    .mapping_for::<portable_build::ResultValues>()
                    .lower(
                        &mut (),
                        JavaResultInput::Ok {
                            value: self.value(value, *ok)?,
                            result: ty,
                        },
                    )
            }
            CoreValue::Err(value) => {
                let CoreType::Result { error, .. } = self
                    .core
                    .types()
                    .get(expected)
                    .expect("verified result type")
                else {
                    return Err(vec![diagnostic("error value does not have a result type")]);
                };
                self.features
                    .mapping_for::<portable_build::ResultValues>()
                    .lower(
                        &mut (),
                        JavaResultInput::Err {
                            value: self.value(value, *error)?,
                            result: ty,
                        },
                    )
            }
            CoreValue::Record { record, fields } => {
                self.construct_value(self.records[record], fields, ty)
            }
            CoreValue::Enum {
                enumeration,
                variant,
                fields,
            } if self.enum_is_payload_free(*enumeration) => {
                if !fields.is_empty() {
                    return Err(vec![diagnostic(
                        "payload-free Java enum value cannot contain fields",
                    )]);
                }
                self.enum_variant_expr(*enumeration, *variant)
            }
            CoreValue::Enum {
                variant, fields, ..
            } => self.construct_value(self.variants[variant], fields, ty),
        }
    }

    fn construct_value(
        &self,
        owner: GeneratedTypeId,
        fields: &[CoreValueField],
        result_type: JavaType,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let arguments = fields
            .iter()
            .map(|field| {
                let metadata = self.core.field(field.field).expect("verified field");
                self.value(&field.value, metadata.ty)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let owner_type = JavaType::Reference(JavaTypeName::Generated(owner));
        let created = JavaExpr {
            ty: owner_type.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::New {
                constructor: JavaConstructorRef::Generated {
                    owner,
                    parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
                },
                arguments,
            },
        };
        if owner_type == result_type {
            Ok(created)
        } else {
            Ok(JavaExpr {
                ty: result_type.clone(),
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Cast {
                    target: result_type,
                    value: Box::new(created),
                },
            })
        }
    }

    fn constant_expr(
        &self,
        value: &CoreConstantExpr,
        expected: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let ty = self.ty(expected)?;
        match &value.kind {
            CoreConstantExprKind::Literal(value) => self.value(value, expected),
            CoreConstantExprKind::Constant(id) => match self
                .features
                .mapping_for::<portable_build::Constants>()
                .lower(
                    &mut (),
                    JavaConstantsInput::Reference {
                        symbol: self.constants[id],
                        result: ty,
                    },
                )? {
                JavaConstantsNode::Expression(value) => Ok(value),
                JavaConstantsNode::Declaration(_) => Err(vec![diagnostic(
                    "Java Constants mapping returned a declaration for a reference",
                )]),
            },
            CoreConstantExprKind::Record { record, fields } => {
                let expressions = fields
                    .iter()
                    .map(|field| {
                        let metadata = self.core.field(field.field).expect("verified field");
                        self.constant_expr(&field.value, metadata.ty)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.construct_java_values(self.records[record], &expressions, ty)
            }
            CoreConstantExprKind::Enum {
                enumeration,
                variant,
                fields,
            } if self.enum_is_payload_free(*enumeration) => {
                if !fields.is_empty() {
                    return Err(vec![diagnostic(
                        "payload-free Java enum constant cannot contain fields",
                    )]);
                }
                self.enum_variant_expr(*enumeration, *variant)
            }
            CoreConstantExprKind::Enum {
                variant, fields, ..
            } => {
                let expressions = fields
                    .iter()
                    .map(|field| {
                        let metadata = self.core.field(field.field).expect("verified field");
                        self.constant_expr(&field.value, metadata.ty)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.construct_java_values(self.variants[variant], &expressions, ty)
            }
            CoreConstantExprKind::Some(value) => {
                let CoreType::Option(inner) =
                    self.core.types().get(expected).expect("verified option")
                else {
                    return Err(vec![diagnostic("constant some has wrong type")]);
                };
                Ok(runtime_call(
                    JavaRuntimeCallable::OptionSome,
                    vec![self.constant_expr(value, *inner)?],
                    ty,
                ))
            }
            CoreConstantExprKind::None { .. } => {
                Ok(runtime_call(JavaRuntimeCallable::OptionNone, vec![], ty))
            }
            CoreConstantExprKind::Ok { value, .. } => {
                let CoreType::Result { ok, .. } =
                    self.core.types().get(expected).expect("verified result")
                else {
                    return Err(vec![diagnostic("constant ok has wrong type")]);
                };
                Ok(runtime_call(
                    JavaRuntimeCallable::ValueResultOk,
                    vec![self.constant_expr(value, *ok)?],
                    ty,
                ))
            }
            CoreConstantExprKind::Err { value, .. } => {
                let CoreType::Result { error, .. } =
                    self.core.types().get(expected).expect("verified result")
                else {
                    return Err(vec![diagnostic("constant error has wrong type")]);
                };
                Ok(runtime_call(
                    JavaRuntimeCallable::ValueResultErr,
                    vec![self.constant_expr(value, *error)?],
                    ty,
                ))
            }
            CoreConstantExprKind::List { element, elements } => {
                let values = elements
                    .iter()
                    .map(|value| self.constant_expr(value, *element))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(known_generic_call(JavaKnownCallable::ListOf, values, ty))
            }
            CoreConstantExprKind::Intrinsic(value) => self.constant_intrinsic(value, expected),
        }
    }

    fn construct_java_values(
        &self,
        owner: GeneratedTypeId,
        fields: &[JavaExpr],
        result_type: JavaType,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        self.lower_record_expr(JavaRecordsInput::Construction {
            owner,
            arguments: fields.to_vec(),
            result: result_type,
        })
    }

    fn test_declaration(&self, class_name: &str) -> Result<JavaTypeDeclaration, Vec<Diagnostic>> {
        let expected_test_count = i32::try_from(self.core.tests().len())
            .map_err(|_| vec![diagnostic("Java conformance test count exceeds i32")])?;
        let mut cases = Vec::with_capacity(self.core.tests().len());
        for (index, test) in self.core.tests().iter().enumerate() {
            let actual = match &test.invocation {
                CoreTestInvocation::Function {
                    function,
                    arguments,
                } => {
                    let function_value = self.core.function(*function).expect("verified function");
                    let arguments = arguments
                        .iter()
                        .map(|value| self.typed_value(value))
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = self.poly_result_type(function_value.return_type)?;
                    let signature = JavaMethodSignature {
                        receiver: None,
                        parameters: function_value
                            .parameters
                            .iter()
                            .map(|parameter| self.ty(parameter.ty))
                            .collect::<Result<Vec<_>, _>>()?,
                        result: result.clone(),
                        checked_exceptions: vec![],
                        nullable_result: false,
                        pure: true,
                    };
                    JavaExpr {
                        ty: result,
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Call {
                            callable: JavaCallableRef::Generated {
                                symbol: self.functions[function],
                                signature,
                            },
                            receiver: None,
                            arguments,
                        },
                    }
                }
                CoreTestInvocation::Method {
                    method,
                    receiver,
                    arguments,
                    ..
                } => {
                    let method_value = self
                        .core
                        .implementation_method(*method)
                        .expect("verified method");
                    let interface_method = self
                        .core
                        .interface_method(method_value.interface_method)
                        .expect("verified interface method");
                    let receiver = self.typed_value(receiver)?;
                    let arguments = arguments
                        .iter()
                        .map(|value| self.typed_value(value))
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = self.poly_result_type(method_value.return_type)?;
                    self.lower_interface_expr(JavaInterfacesInput::ConcreteCall(Box::new(
                        JavaConcreteInterfaceCallInput {
                            receiver,
                            interface_method_name: interface_method.header.name.clone(),
                            arguments,
                            result,
                            method: *method,
                        },
                    )))?
                }
            };
            let expected = match &test.expected {
                portable_core_ir::CoreExpectedOutcome::Value(value) => {
                    JavaPortableTestExpectation::Value(self.typed_value(value)?)
                }
                portable_core_ir::CoreExpectedOutcome::Error(value) => {
                    JavaPortableTestExpectation::Error(self.typed_value(value)?)
                }
            };
            let JavaPortableTestsNode::Case(statements) = self
                .features
                .mapping_for::<portable_build::PortableTests>()
                .lower(
                    &mut (),
                    JavaPortableTestsInput::Case(Box::new(JavaPortableTestCaseInput {
                        index,
                        name: test.header.name.clone(),
                        actual,
                        expected,
                    })),
                )?
            else {
                return Err(vec![diagnostic(
                    "Java PortableTests mapping returned a harness for a case",
                )]);
            };
            cases.push(statements);
        }
        match self
            .features
            .mapping_for::<portable_build::PortableTests>()
            .lower(
                &mut (),
                JavaPortableTestsInput::Harness(JavaPortableTestHarnessInput {
                    class_name: class_name.to_owned(),
                    cases,
                    expected_test_count,
                }),
            )? {
            JavaPortableTestsNode::Harness(declaration) => Ok(declaration),
            JavaPortableTestsNode::Case(_) => Err(vec![diagnostic(
                "Java PortableTests mapping returned a case for a harness",
            )]),
        }
    }

    fn typed_value(&self, value: &CoreTypedValue) -> Result<JavaExpr, Vec<Diagnostic>> {
        self.value(&value.value, value.ty)
    }

    fn match_plan(
        &self,
        value: CoreExprId,
        arms: &[CoreMatchArm],
        result: CoreTypeId,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let matched_expression = self
            .core
            .expressions()
            .get(value)
            .expect("verified match expression");
        if let Some(CoreType::Enum(enumeration)) = self.core.types().get(matched_expression.ty)
            && self.enum_is_payload_free(*enumeration)
            && arms
                .iter()
                .all(|arm| matches!(arm.pattern, CorePattern::EnumVariant { .. }))
        {
            return self.payload_free_enum_match_plan(
                value,
                arms,
                result,
                callable_return,
                *enumeration,
            );
        }
        let matched = self.expr_plan(value, callable_return)?;
        let matched_type = matched.value.ty.clone();
        let result_type = self.ty(result)?;
        let (matched_name, matched_local) = self.temporary("matchValue", matched_type.clone());
        let (result_name, result_local) = self.temporary("matchResult", result_type.clone());
        let arms = arms
            .iter()
            .enumerate()
            .map(|(index, arm)| {
                Ok(JavaMatchArmInput {
                    pattern: self.pattern(&arm.pattern, matched_local.clone(), index)?,
                    body: self.block(
                        arm.body,
                        BlockMode::AssignResult {
                            target: Box::new(result_local.clone()),
                        },
                        callable_return,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
        match self
            .features
            .mapping_for::<portable_build::PatternMatching>()
            .lower(
                &mut (),
                JavaPatternMatchingInput::Match(Box::new(JavaMatchInput {
                    prefix: matched.statements,
                    matched: matched.value,
                    matched_name,
                    result_name,
                    result_type,
                    arms,
                })),
            )? {
            JavaPatternMatchingNode::Match(plan) => {
                let JavaPatternMatchPlan { statements, value } = *plan;
                Ok(ExprPlan { statements, value })
            }
            JavaPatternMatchingNode::Pattern(_) => Err(vec![diagnostic(
                "Java PatternMatching mapping returned a pattern for a match",
            )]),
        }
    }

    fn payload_free_enum_match_plan(
        &self,
        value: CoreExprId,
        arms: &[CoreMatchArm],
        result: CoreTypeId,
        callable_return: CoreTypeId,
        enumeration: CoreEnumId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let matched = self.expr_plan(value, callable_return)?;
        let result_type = self.ty(result)?;
        let (matched_name, matched_local) = self.temporary("matchValue", matched.value.ty.clone());
        let (result_name, result_local) = self.temporary("matchResult", result_type.clone());
        let mut statements = matched.statements;
        statements.push(JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: matched.value.ty.clone(),
            name: matched_name,
            value: Some(matched.value),
        });
        statements.push(JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty: result_type,
            name: result_name,
            value: None,
        });
        let lowered_arms = arms
            .iter()
            .map(|arm| {
                let CorePattern::EnumVariant {
                    variant, bindings, ..
                } = &arm.pattern
                else {
                    unreachable!("payload-free enum match precondition")
                };
                if !bindings.is_empty() {
                    return Err(vec![diagnostic(
                        "payload-free Java enum patterns cannot bind fields",
                    )]);
                }
                Ok(JavaEnumBranchInput {
                    variant: self.enum_values[variant],
                    body: self.block(
                        arm.body,
                        BlockMode::AssignResult {
                            target: Box::new(result_local.clone()),
                        },
                        callable_return,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
        let enumeration_value = self.core.enumeration(enumeration).expect("verified enum");
        let branch = self.features.mapping_for::<portable_build::Enums>().lower(
            &mut (),
            JavaEnumsInput::Branch {
                selector: Box::new(matched_local),
                enumeration: self.enums[&enumeration],
                declared_variants: enumeration_value
                    .variants
                    .iter()
                    .map(|variant| self.enum_values[variant])
                    .collect(),
                arms: lowered_arms,
            },
        )?;
        match branch {
            JavaEnumsNode::Statement(branch) => statements.push(*branch),
            JavaEnumsNode::Declaration(_) | JavaEnumsNode::Expression(_) => {
                return Err(vec![diagnostic(
                    "Java Enums mapping returned a value for exhaustive branching",
                )]);
            }
        }
        Ok(ExprPlan {
            statements,
            value: result_local,
        })
    }

    fn pattern(
        &self,
        pattern: &CorePattern,
        matched: JavaExpr,
        index: usize,
    ) -> Result<JavaLoweredPattern, Vec<Diagnostic>> {
        let input = match pattern {
            CorePattern::Wildcard { .. } => JavaPatternInput::Wildcard,
            CorePattern::Bool { value, .. } => JavaPatternInput::Bool {
                matched: Box::new(matched),
                value: *value,
            },
            CorePattern::EnumVariant {
                variant, bindings, ..
            } => {
                let variant_type =
                    JavaType::Reference(JavaTypeName::Generated(self.variants[variant]));
                let bindings = bindings
                    .iter()
                    .map(|binding| {
                        let local_value = self
                            .core
                            .local(binding.binding)
                            .expect("verified pattern local");
                        let field = self.core.field(binding.field).expect("verified field");
                        Ok(JavaPatternFieldBindingInput {
                            binding_name: local_value.name.clone(),
                            binding_type: self.ty(local_value.ty)?,
                            field_name: field.header.name.clone(),
                            field_type: self.ty(field.ty)?,
                            field: binding.field,
                        })
                    })
                    .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
                JavaPatternInput::EnumVariant {
                    matched: Box::new(matched),
                    variant_type,
                    variant_name: format!("matchedVariant{index}"),
                    bindings,
                }
            }
            CorePattern::None { .. } => JavaPatternInput::None {
                matched: Box::new(matched),
            },
            CorePattern::Some { binding, .. } => {
                let local_value = self.core.local(*binding).expect("verified pattern local");
                JavaPatternInput::Some {
                    matched: Box::new(matched),
                    binding_name: local_value.name.clone(),
                    binding_type: self.ty(local_value.ty)?,
                }
            }
            CorePattern::Ok { binding, .. } => {
                let local_value = self.core.local(*binding).expect("verified pattern local");
                JavaPatternInput::Ok {
                    matched: Box::new(matched),
                    binding_name: local_value.name.clone(),
                    binding_type: self.ty(local_value.ty)?,
                }
            }
            CorePattern::Err { binding, .. } => {
                let local_value = self.core.local(*binding).expect("verified pattern local");
                JavaPatternInput::Err {
                    matched: Box::new(matched),
                    binding_name: local_value.name.clone(),
                    binding_type: self.ty(local_value.ty)?,
                }
            }
        };
        match self
            .features
            .mapping_for::<portable_build::PatternMatching>()
            .lower(&mut (), JavaPatternMatchingInput::Pattern(Box::new(input)))?
        {
            JavaPatternMatchingNode::Pattern(pattern) => Ok(*pattern),
            JavaPatternMatchingNode::Match(_) => Err(vec![diagnostic(
                "Java PatternMatching mapping returned a match for a pattern",
            )]),
        }
    }

    fn intrinsic_plan(
        &self,
        value: &CoreIntrinsicExpr<CoreExprId>,
        result: CoreTypeId,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        if let CoreIntrinsicExpr::Binary {
            operation: operation @ (CoreBinaryIntrinsic::BoolAnd | CoreBinaryIntrinsic::BoolOr),
            left,
            right,
        } = value
        {
            return self.short_circuit_boolean(*operation, *left, *right, callable_return);
        }

        let mut statements = Vec::new();
        let mapped = match value {
            CoreIntrinsicExpr::Unary { operation, operand } => {
                let operand = self.stabilize_plan(
                    self.expr_plan(*operand, callable_return)?,
                    "intrinsicOperand",
                );
                statements.extend(operand.statements);
                CoreIntrinsicExpr::Unary {
                    operation: *operation,
                    operand: operand.value,
                }
            }
            CoreIntrinsicExpr::Binary {
                operation,
                left,
                right,
            } => {
                let left = self
                    .stabilize_plan(self.expr_plan(*left, callable_return)?, "intrinsicOperand");
                statements.extend(left.statements);
                let right = self
                    .stabilize_plan(self.expr_plan(*right, callable_return)?, "intrinsicOperand");
                statements.extend(right.statements);
                CoreIntrinsicExpr::Binary {
                    operation: *operation,
                    left: left.value,
                    right: right.value,
                }
            }
            CoreIntrinsicExpr::Ternary {
                operation,
                first,
                second,
                third,
            } => {
                let first = self
                    .stabilize_plan(self.expr_plan(*first, callable_return)?, "intrinsicOperand");
                statements.extend(first.statements);
                let second = self.stabilize_plan(
                    self.expr_plan(*second, callable_return)?,
                    "intrinsicOperand",
                );
                statements.extend(second.statements);
                let third = self
                    .stabilize_plan(self.expr_plan(*third, callable_return)?, "intrinsicOperand");
                statements.extend(third.statements);
                CoreIntrinsicExpr::Ternary {
                    operation: *operation,
                    first: first.value,
                    second: second.value,
                    third: third.value,
                }
            }
            CoreIntrinsicExpr::Variadic {
                operation,
                arguments,
            } => {
                let (argument_statements, arguments) =
                    self.expr_list(arguments, callable_return)?;
                statements.extend(argument_statements);
                CoreIntrinsicExpr::Variadic {
                    operation: *operation,
                    arguments,
                }
            }
        };
        match self.intrinsic_java(mapped, self.ty(result)?)? {
            JavaIntrinsicExpr::Direct(value) => Ok(ExprPlan { statements, value }),
            JavaIntrinsicExpr::Fallible { call, value_type } => {
                self.propagate_call(statements, call, value_type, callable_return)
            }
        }
    }

    fn constant_intrinsic(
        &self,
        value: &CoreIntrinsicExpr<CoreConstantExpr>,
        result: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let mapped = match value {
            CoreIntrinsicExpr::Unary { operation, operand } => CoreIntrinsicExpr::Unary {
                operation: *operation,
                operand: self.constant_untyped(operand)?.0,
            },
            CoreIntrinsicExpr::Binary {
                operation,
                left,
                right,
            } => CoreIntrinsicExpr::Binary {
                operation: *operation,
                left: self.constant_untyped(left)?.0,
                right: self.constant_untyped(right)?.0,
            },
            CoreIntrinsicExpr::Ternary {
                operation,
                first,
                second,
                third,
            } => CoreIntrinsicExpr::Ternary {
                operation: *operation,
                first: self.constant_untyped(first)?.0,
                second: self.constant_untyped(second)?.0,
                third: self.constant_untyped(third)?.0,
            },
            CoreIntrinsicExpr::Variadic {
                operation,
                arguments,
            } => CoreIntrinsicExpr::Variadic {
                operation: *operation,
                arguments: arguments
                    .iter()
                    .map(|value| self.constant_untyped(value).map(|value| value.0))
                    .collect::<Result<Vec<_>, _>>()?,
            },
        };
        match self.intrinsic_java(mapped, self.ty(result)?)? {
            JavaIntrinsicExpr::Direct(value) => Ok(value),
            JavaIntrinsicExpr::Fallible { .. } => Err(vec![Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                "Java constants cannot contain a fallible intrinsic",
                source("constant-fallible-intrinsic"),
            )]),
        }
    }

    fn constant_untyped(
        &self,
        value: &CoreConstantExpr,
    ) -> Result<(JavaExpr, JavaType), Vec<Diagnostic>> {
        let core_type = self.constant_type(value)?;
        let java_type = self.ty(core_type)?;
        Ok((self.constant_expr(value, core_type)?, java_type))
    }

    fn constant_type(&self, value: &CoreConstantExpr) -> Result<CoreTypeId, Vec<Diagnostic>> {
        match &value.kind {
            CoreConstantExprKind::Literal(value) => self.constant_value_type(value),
            CoreConstantExprKind::Constant(id) => Ok(self
                .core
                .constant(*id)
                .expect("verified constant reference")
                .ty),
            CoreConstantExprKind::Record { record, .. } => {
                self.find_type(&CoreType::Record(*record))
            }
            CoreConstantExprKind::Enum { enumeration, .. } => {
                self.find_type(&CoreType::Enum(*enumeration))
            }
            CoreConstantExprKind::Some(inner) => {
                let inner = self.constant_type(inner)?;
                self.find_type(&CoreType::Option(inner))
            }
            CoreConstantExprKind::None { inner } => self.find_type(&CoreType::Option(*inner)),
            CoreConstantExprKind::Ok { value, error } => {
                let ok = self.constant_type(value)?;
                self.find_type(&CoreType::Result { ok, error: *error })
            }
            CoreConstantExprKind::Err { value, ok } => {
                let error = self.constant_type(value)?;
                self.find_type(&CoreType::Result { ok: *ok, error })
            }
            CoreConstantExprKind::List { element, .. } => self.find_type(&CoreType::List(*element)),
            CoreConstantExprKind::Intrinsic(intrinsic) => self.constant_intrinsic_type(intrinsic),
        }
    }

    fn constant_value_type(&self, value: &CoreValue) -> Result<CoreTypeId, Vec<Diagnostic>> {
        match value {
            CoreValue::Unit => self.find_type(&CoreType::Unit),
            CoreValue::Bool(_) => self.find_type(&CoreType::Bool),
            CoreValue::I32(_) => self.find_type(&CoreType::I32),
            CoreValue::I64(_) => self.find_type(&CoreType::I64),
            CoreValue::F64(_) => self.find_type(&CoreType::F64),
            CoreValue::Char(_) => self.find_type(&CoreType::Char),
            CoreValue::String(_) => self.find_type(&CoreType::String),
            CoreValue::Bytes(_) => self.find_type(&CoreType::Bytes),
            CoreValue::List(values) if !values.is_empty() => {
                let element = self.constant_value_type(&values[0])?;
                self.find_type(&CoreType::List(element))
            }
            CoreValue::Some(value) => {
                let inner = self.constant_value_type(value)?;
                self.find_type(&CoreType::Option(inner))
            }
            CoreValue::Record { record, .. } => self.find_type(&CoreType::Record(*record)),
            CoreValue::Enum { enumeration, .. } => self.find_type(&CoreType::Enum(*enumeration)),
            CoreValue::None | CoreValue::List(_) | CoreValue::Ok(_) | CoreValue::Err(_) => {
                Err(vec![Diagnostic::error(
                    DiagnosticCode::InvalidStructure,
                    "verified nested constant value has no inferable declared Core type",
                    source("constant-type"),
                )])
            }
        }
    }

    fn constant_intrinsic_type(
        &self,
        value: &CoreIntrinsicExpr<CoreConstantExpr>,
    ) -> Result<CoreTypeId, Vec<Diagnostic>> {
        use CoreBinaryIntrinsic as B;
        use CoreUnaryIntrinsic as U;

        let result = match value {
            CoreIntrinsicExpr::Unary { operation, operand } => {
                let operand = self.constant_type(operand)?;
                match operation {
                    U::BoolNot
                    | U::FloatIsNaN
                    | U::FloatIsNegativeZero
                    | U::StringIsEmpty
                    | U::BytesIsEmpty
                    | U::ListIsEmpty
                    | U::OptionIsSome
                    | U::OptionIsNone
                    | U::ResultIsOk
                    | U::ResultIsErr => CoreType::Bool,
                    U::IntNegChecked | U::IntNegWrapping | U::IntBitNot => {
                        return Ok(operand);
                    }
                    U::FloatNeg | U::FloatTrunc | U::FloatAbs => CoreType::F64,
                    U::StringScalarLength
                    | U::StringUtf16Length
                    | U::BytesLength
                    | U::ListLength
                    | U::WidenI32ToI64 => CoreType::I64,
                    U::NarrowI64ToI32Checked => CoreType::I32,
                    U::StringToUtf8 => CoreType::Bytes,
                    U::StringFromUtf8Checked => CoreType::String,
                }
            }
            CoreIntrinsicExpr::Binary {
                operation,
                left,
                right,
            } => {
                let left = self.constant_type(left)?;
                let right = self.constant_type(right)?;
                match operation {
                    B::BoolAnd
                    | B::BoolOr
                    | B::Equal
                    | B::NotEqual
                    | B::Less
                    | B::LessEqual
                    | B::Greater
                    | B::GreaterEqual
                    | B::StringContains
                    | B::StringStartsWith
                    | B::StringEndsWith
                    | B::ListContains => CoreType::Bool,
                    B::IntAddChecked
                    | B::IntSubChecked
                    | B::IntMulChecked
                    | B::IntDivChecked
                    | B::IntRemChecked
                    | B::IntAddWrapping
                    | B::IntSubWrapping
                    | B::IntMulWrapping
                    | B::IntBitAnd
                    | B::IntBitOr
                    | B::IntBitXor
                    | B::IntShiftLeftChecked
                    | B::IntShiftRightChecked
                    | B::ListAppend
                    | B::ListConcat => return Ok(left),
                    B::FloatAdd | B::FloatSub | B::FloatMul | B::FloatDiv | B::FloatRemTrunc => {
                        CoreType::F64
                    }
                    B::StringConcat
                    | B::StringStripPrefix
                    | B::StringTruncateUtf8Bytes
                    | B::StringTrimStart
                    | B::StringTrimEnd => CoreType::String,
                    B::BytesConcat => CoreType::Bytes,
                    B::StringIndexOfLiteral | B::ListIndexOf => {
                        let i64_type = self.find_type(&CoreType::I64)?;
                        return self.find_type(&CoreType::Option(i64_type));
                    }
                    B::ListGetChecked => match self.core.types().get(left) {
                        Some(CoreType::List(inner)) => return Ok(*inner),
                        _ => {
                            return Err(vec![diagnostic(
                                "verified list-get constant operand is not a list",
                            )]);
                        }
                    },
                    B::OptionUnwrapOr => return Ok(right),
                }
            }
            CoreIntrinsicExpr::Ternary { operation, .. } => match operation {
                CoreTernaryIntrinsic::StringSliceScalars
                | CoreTernaryIntrinsic::StringReplaceAll => CoreType::String,
                CoreTernaryIntrinsic::BytesReplaceAll => CoreType::Bytes,
            },
            CoreIntrinsicExpr::Variadic { operation, .. } => match operation {
                CoreVariadicIntrinsic::StringReplaceMany => CoreType::String,
            },
        };
        self.find_type(&result)
    }

    fn find_type(&self, wanted: &CoreType) -> Result<CoreTypeId, Vec<Diagnostic>> {
        self.core
            .types()
            .iter()
            .find_map(|(id, value)| (value == wanted).then_some(id))
            .ok_or_else(|| vec![diagnostic("verified CoreIR type was not interned")])
    }

    fn intrinsic_java(
        &self,
        value: CoreIntrinsicExpr<JavaExpr>,
        result: JavaType,
    ) -> Result<JavaIntrinsicExpr, Vec<Diagnostic>> {
        if let CoreIntrinsicExpr::Binary {
            operation,
            left,
            right,
        } = &value
            && matches!(
                operation,
                CoreBinaryIntrinsic::Equal | CoreBinaryIntrinsic::NotEqual
            )
            && let Some(enumeration) = self.payload_free_java_enum_type(&left.ty)
        {
            let JavaEnumsNode::Expression(equal) =
                self.features.mapping_for::<portable_build::Enums>().lower(
                    &mut (),
                    JavaEnumsInput::Equality {
                        enumeration,
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    },
                )?
            else {
                return Err(vec![diagnostic(
                    "Java Enums mapping returned a non-expression for equality",
                )]);
            };
            return Ok(JavaIntrinsicExpr::Direct(
                if *operation == CoreBinaryIntrinsic::Equal {
                    *equal
                } else {
                    unary(JavaUnaryOperator::Not, *equal, result)
                },
            ));
        }
        let mut context = ();
        match classify_intrinsic(value, result) {
            JavaIntrinsicFamily::BooleanLogic(input) => self
                .features
                .mapping_for::<BooleanLogic>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::Equality(input) => self
                .features
                .mapping_for::<Equality>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::Ordering(input) => self
                .features
                .mapping_for::<Ordering>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::CheckedIntegerArithmetic(input) => self
                .features
                .mapping_for::<CheckedIntegerArithmetic>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::WrappingIntegerArithmetic(input) => self
                .features
                .mapping_for::<WrappingIntegerArithmetic>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::FloatingPointArithmetic(input) => self
                .features
                .mapping_for::<FloatingPointArithmetic>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::StringConcatenation(input) => self
                .features
                .mapping_for::<StringConcatenation>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::IntegerBitwise(input) => self
                .features
                .mapping_for::<IntegerBitwise>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::CheckedIntegerShifts(input) => self
                .features
                .mapping_for::<CheckedIntegerShifts>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::FloatingPointInspection(input) => self
                .features
                .mapping_for::<FloatingPointInspection>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::StringInspection(input) => self
                .features
                .mapping_for::<StringInspection>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::StringTransformation(input) => self
                .features
                .mapping_for::<StringTransformation>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::BytesOperations(input) => self
                .features
                .mapping_for::<BytesOperations>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::ListOperations(input) => self
                .features
                .mapping_for::<ListOperations>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::OptionOperations(input) => self
                .features
                .mapping_for::<OptionOperations>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::ResultOperations(input) => self
                .features
                .mapping_for::<ResultOperations>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::IntegerConversions(input) => self
                .features
                .mapping_for::<IntegerConversions>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::Utf8Conversions(input) => self
                .features
                .mapping_for::<Utf8Conversions>()
                .lower(&mut context, input),
        }
    }

    fn payload_free_java_enum_type(&self, ty: &JavaType) -> Option<GeneratedTypeId> {
        let JavaType::Reference(JavaTypeName::Generated(generated)) = ty else {
            return None;
        };
        self.enums.iter().find_map(|(enumeration, candidate)| {
            (candidate == generated && self.enum_is_payload_free(*enumeration))
                .then_some(*generated)
        })
    }

    fn intrinsic_java_raw(
        value: CoreIntrinsicExpr<JavaExpr>,
        result: JavaType,
    ) -> Result<JavaIntrinsicExpr, Vec<Diagnostic>> {
        Ok(match value {
            CoreIntrinsicExpr::Unary { operation, operand } => {
                Self::unary_intrinsic(operation, operand, result)?
            }
            CoreIntrinsicExpr::Binary {
                operation,
                left,
                right,
            } => Self::binary_intrinsic(operation, left, right, result)?,
            CoreIntrinsicExpr::Ternary {
                operation,
                first,
                second,
                third,
            } => JavaIntrinsicExpr::Direct(match operation {
                CoreTernaryIntrinsic::StringSliceScalars => runtime_call(
                    JavaRuntimeCallable::StringSliceScalars,
                    vec![first, second, third],
                    result,
                ),
                CoreTernaryIntrinsic::StringReplaceAll => runtime_call(
                    JavaRuntimeCallable::StringReplaceAll,
                    vec![first, second, third],
                    result,
                ),
                CoreTernaryIntrinsic::BytesReplaceAll => runtime_call(
                    JavaRuntimeCallable::BytesReplaceAll,
                    vec![first, second, third],
                    result,
                ),
            }),
            CoreIntrinsicExpr::Variadic {
                operation: CoreVariadicIntrinsic::StringReplaceMany,
                arguments,
            } => {
                let mut arguments = arguments.into_iter();
                let source = arguments
                    .next()
                    .ok_or_else(|| vec![diagnostic("replace-many source missing")])?;
                let pairs = arguments.collect::<Vec<_>>();
                let pair_list = JavaType::generic(
                    JavaKnownType::List,
                    vec![JavaType::known(JavaKnownType::String)],
                );
                JavaIntrinsicExpr::Direct(runtime_call(
                    JavaRuntimeCallable::StringReplaceMany,
                    vec![
                        source,
                        known_generic_call(JavaKnownCallable::ListOf, pairs, pair_list),
                    ],
                    result,
                ))
            }
        })
    }

    fn unary_intrinsic(
        operation: CoreUnaryIntrinsic,
        operand: JavaExpr,
        result: JavaType,
    ) -> Result<JavaIntrinsicExpr, Vec<Diagnostic>> {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let value = match operation {
            CoreUnaryIntrinsic::BoolNot => unary(JavaUnaryOperator::Not, operand, result),
            CoreUnaryIntrinsic::IntNegChecked => {
                let callable = match operand.ty {
                    JavaType::Primitive(JavaPrimitive::Int) => JavaRuntimeCallable::CheckedNegI32,
                    JavaType::Primitive(JavaPrimitive::Long) => JavaRuntimeCallable::CheckedNegI64,
                    _ => {
                        return Err(vec![diagnostic(
                            "checked negation requires a Java int or long",
                        )]);
                    }
                };
                return Ok(runtime_fallible(callable, vec![operand], result));
            }
            CoreUnaryIntrinsic::IntNegWrapping | CoreUnaryIntrinsic::FloatNeg => {
                unary(JavaUnaryOperator::Negate, operand, result)
            }
            CoreUnaryIntrinsic::IntBitNot => unary(JavaUnaryOperator::BitNot, operand, result),
            CoreUnaryIntrinsic::FloatTrunc => {
                runtime_call(JavaRuntimeCallable::FloatTrunc, vec![operand], result)
            }
            CoreUnaryIntrinsic::FloatIsNaN => {
                known_call(JavaKnownCallable::DoubleIsNaN, vec![operand])
            }
            CoreUnaryIntrinsic::FloatIsNegativeZero => runtime_call(
                JavaRuntimeCallable::FloatIsNegativeZero,
                vec![operand],
                result,
            ),
            CoreUnaryIntrinsic::FloatAbs => {
                runtime_call(JavaRuntimeCallable::FloatAbs, vec![operand], result)
            }
            CoreUnaryIntrinsic::StringScalarLength => {
                return Ok(runtime_fallible(
                    JavaRuntimeCallable::ScalarLength,
                    vec![operand],
                    result,
                ));
            }
            CoreUnaryIntrinsic::StringUtf16Length => {
                let length = member_call(
                    operand,
                    "length",
                    vec![],
                    JavaType::primitive(JavaPrimitive::Int),
                    JavaMemberOrigin::Known(JavaKnownMethod::StringLength),
                );
                JavaExpr {
                    ty: result.clone(),
                    precedence: JavaPrecedence::Unary,
                    kind: JavaExprKind::Cast {
                        target: result,
                        value: Box::new(length),
                    },
                }
            }
            CoreUnaryIntrinsic::StringIsEmpty => member_call(
                operand,
                "isEmpty",
                vec![],
                result,
                JavaMemberOrigin::Known(JavaKnownMethod::StringIsEmpty),
            ),
            CoreUnaryIntrinsic::BytesLength => {
                runtime_call(JavaRuntimeCallable::BytesLength, vec![operand], result)
            }
            CoreUnaryIntrinsic::BytesIsEmpty => {
                runtime_call(JavaRuntimeCallable::BytesIsEmpty, vec![operand], result)
            }
            CoreUnaryIntrinsic::ListLength => {
                runtime_call(JavaRuntimeCallable::ListLength, vec![operand], result)
            }
            CoreUnaryIntrinsic::ListIsEmpty => {
                runtime_call(JavaRuntimeCallable::ListIsEmpty, vec![operand], result)
            }
            CoreUnaryIntrinsic::OptionIsSome => {
                runtime_call(JavaRuntimeCallable::OptionIsSome, vec![operand], result)
            }
            CoreUnaryIntrinsic::OptionIsNone => {
                let some = runtime_call(
                    JavaRuntimeCallable::OptionIsSome,
                    vec![operand],
                    boolean.clone(),
                );
                unary(JavaUnaryOperator::Not, some, result)
            }
            CoreUnaryIntrinsic::ResultIsOk => {
                runtime_call(JavaRuntimeCallable::ValueResultIsOk, vec![operand], result)
            }
            CoreUnaryIntrinsic::ResultIsErr => {
                let ok = runtime_call(
                    JavaRuntimeCallable::ValueResultIsOk,
                    vec![operand],
                    boolean.clone(),
                );
                unary(JavaUnaryOperator::Not, ok, result)
            }
            CoreUnaryIntrinsic::WidenI32ToI64 => JavaExpr {
                ty: result.clone(),
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Cast {
                    target: result,
                    value: Box::new(operand),
                },
            },
            CoreUnaryIntrinsic::NarrowI64ToI32Checked => {
                return Ok(runtime_fallible(
                    JavaRuntimeCallable::NarrowI64ToI32,
                    vec![operand],
                    result,
                ));
            }
            CoreUnaryIntrinsic::StringToUtf8 => {
                runtime_call(JavaRuntimeCallable::StringToUtf8, vec![operand], result)
            }
            CoreUnaryIntrinsic::StringFromUtf8Checked => {
                return Ok(runtime_fallible(
                    JavaRuntimeCallable::StringFromUtf8,
                    vec![operand],
                    result,
                ));
            }
        };
        Ok(JavaIntrinsicExpr::Direct(value))
    }

    fn binary_intrinsic(
        operation: CoreBinaryIntrinsic,
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    ) -> Result<JavaIntrinsicExpr, Vec<Diagnostic>> {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let numeric_width = match left.ty {
            JavaType::Primitive(JavaPrimitive::Int) => Some(false),
            JavaType::Primitive(JavaPrimitive::Long) => Some(true),
            _ => None,
        };
        let direct = |operator| binary(operator, left.clone(), right.clone(), result.clone());
        let value = match operation {
            CoreBinaryIntrinsic::BoolAnd => direct(JavaBinaryOperator::LogicalAnd),
            CoreBinaryIntrinsic::BoolOr => direct(JavaBinaryOperator::LogicalOr),
            CoreBinaryIntrinsic::Equal | CoreBinaryIntrinsic::NotEqual => {
                let equal = runtime_call(
                    JavaRuntimeCallable::SemanticEqual,
                    vec![left, right],
                    boolean.clone(),
                );
                if operation == CoreBinaryIntrinsic::Equal {
                    equal
                } else {
                    unary(JavaUnaryOperator::Not, equal, result)
                }
            }
            CoreBinaryIntrinsic::Less
            | CoreBinaryIntrinsic::LessEqual
            | CoreBinaryIntrinsic::Greater
            | CoreBinaryIntrinsic::GreaterEqual => {
                let operator = match operation {
                    CoreBinaryIntrinsic::Less => JavaBinaryOperator::Less,
                    CoreBinaryIntrinsic::LessEqual => JavaBinaryOperator::LessEqual,
                    CoreBinaryIntrinsic::Greater => JavaBinaryOperator::Greater,
                    CoreBinaryIntrinsic::GreaterEqual => JavaBinaryOperator::GreaterEqual,
                    _ => unreachable!(),
                };
                if left.ty == JavaType::known(JavaKnownType::String) {
                    binary(
                        operator,
                        runtime_call(
                            JavaRuntimeCallable::CompareScalarStrings,
                            vec![left, right],
                            JavaType::primitive(JavaPrimitive::Int),
                        ),
                        i32_literal(0),
                        result,
                    )
                } else if left.ty == JavaType::known(JavaKnownType::RuntimeScalar) {
                    binary(
                        operator,
                        member_call(
                            left,
                            "value",
                            vec![],
                            JavaType::primitive(JavaPrimitive::Int),
                            JavaMemberOrigin::Runtime(JavaRuntimeMember::ScalarValue),
                        ),
                        member_call(
                            right,
                            "value",
                            vec![],
                            JavaType::primitive(JavaPrimitive::Int),
                            JavaMemberOrigin::Runtime(JavaRuntimeMember::ScalarValue),
                        ),
                        result,
                    )
                } else {
                    direct(operator)
                }
            }
            CoreBinaryIntrinsic::IntAddChecked
            | CoreBinaryIntrinsic::IntSubChecked
            | CoreBinaryIntrinsic::IntMulChecked
            | CoreBinaryIntrinsic::IntDivChecked
            | CoreBinaryIntrinsic::IntRemChecked => {
                let wide = numeric_width
                    .ok_or_else(|| vec![diagnostic("checked arithmetic requires int or long")])?;
                let callable = match (operation, wide) {
                    (CoreBinaryIntrinsic::IntAddChecked, false) => {
                        JavaRuntimeCallable::CheckedAddI32
                    }
                    (CoreBinaryIntrinsic::IntAddChecked, true) => {
                        JavaRuntimeCallable::CheckedAddI64
                    }
                    (CoreBinaryIntrinsic::IntSubChecked, false) => {
                        JavaRuntimeCallable::CheckedSubI32
                    }
                    (CoreBinaryIntrinsic::IntSubChecked, true) => {
                        JavaRuntimeCallable::CheckedSubI64
                    }
                    (CoreBinaryIntrinsic::IntMulChecked, false) => {
                        JavaRuntimeCallable::CheckedMulI32
                    }
                    (CoreBinaryIntrinsic::IntMulChecked, true) => {
                        JavaRuntimeCallable::CheckedMulI64
                    }
                    (CoreBinaryIntrinsic::IntDivChecked, false) => {
                        JavaRuntimeCallable::CheckedDivI32
                    }
                    (CoreBinaryIntrinsic::IntDivChecked, true) => {
                        JavaRuntimeCallable::CheckedDivI64
                    }
                    (CoreBinaryIntrinsic::IntRemChecked, false) => {
                        JavaRuntimeCallable::CheckedRemI32
                    }
                    (CoreBinaryIntrinsic::IntRemChecked, true) => {
                        JavaRuntimeCallable::CheckedRemI64
                    }
                    _ => unreachable!(),
                };
                return Ok(runtime_fallible(callable, vec![left, right], result));
            }
            CoreBinaryIntrinsic::IntAddWrapping | CoreBinaryIntrinsic::FloatAdd => {
                direct(JavaBinaryOperator::Add)
            }
            CoreBinaryIntrinsic::IntSubWrapping | CoreBinaryIntrinsic::FloatSub => {
                direct(JavaBinaryOperator::Subtract)
            }
            CoreBinaryIntrinsic::IntMulWrapping | CoreBinaryIntrinsic::FloatMul => {
                direct(JavaBinaryOperator::Multiply)
            }
            CoreBinaryIntrinsic::FloatDiv => direct(JavaBinaryOperator::Divide),
            CoreBinaryIntrinsic::FloatRemTrunc => direct(JavaBinaryOperator::Remainder),
            CoreBinaryIntrinsic::IntBitAnd => direct(JavaBinaryOperator::BitAnd),
            CoreBinaryIntrinsic::IntBitOr => direct(JavaBinaryOperator::BitOr),
            CoreBinaryIntrinsic::IntBitXor => direct(JavaBinaryOperator::BitXor),
            CoreBinaryIntrinsic::IntShiftLeftChecked
            | CoreBinaryIntrinsic::IntShiftRightChecked => {
                let wide = numeric_width
                    .ok_or_else(|| vec![diagnostic("checked shift requires int or long")])?;
                let callable = match (operation, wide) {
                    (CoreBinaryIntrinsic::IntShiftLeftChecked, false) => {
                        JavaRuntimeCallable::CheckedShiftLeftI32
                    }
                    (CoreBinaryIntrinsic::IntShiftLeftChecked, true) => {
                        JavaRuntimeCallable::CheckedShiftLeftI64
                    }
                    (CoreBinaryIntrinsic::IntShiftRightChecked, false) => {
                        JavaRuntimeCallable::CheckedShiftRightI32
                    }
                    (CoreBinaryIntrinsic::IntShiftRightChecked, true) => {
                        JavaRuntimeCallable::CheckedShiftRightI64
                    }
                    _ => unreachable!(),
                };
                return Ok(runtime_fallible(callable, vec![left, right], result));
            }
            CoreBinaryIntrinsic::StringConcat => direct(JavaBinaryOperator::Add),
            CoreBinaryIntrinsic::StringIndexOfLiteral => runtime_call(
                JavaRuntimeCallable::StringIndexOfLiteral,
                vec![left, right],
                result,
            ),
            CoreBinaryIntrinsic::StringContains => member_call(
                left,
                "contains",
                vec![right],
                result,
                JavaMemberOrigin::Known(JavaKnownMethod::StringContains),
            ),
            CoreBinaryIntrinsic::StringStartsWith => member_call(
                left,
                "startsWith",
                vec![right],
                result,
                JavaMemberOrigin::Known(JavaKnownMethod::StringStartsWith),
            ),
            CoreBinaryIntrinsic::StringStripPrefix => {
                let starts = member_call(
                    left.clone(),
                    "startsWith",
                    vec![right.clone()],
                    boolean,
                    JavaMemberOrigin::Known(JavaKnownMethod::StringStartsWith),
                );
                let length = member_call(
                    right,
                    "length",
                    vec![],
                    JavaType::primitive(JavaPrimitive::Int),
                    JavaMemberOrigin::Known(JavaKnownMethod::StringLength),
                );
                let stripped = member_call(
                    left.clone(),
                    "substring",
                    vec![length],
                    result.clone(),
                    JavaMemberOrigin::Known(JavaKnownMethod::StringSubstringFrom),
                );
                conditional(starts, stripped, left, result)
            }
            CoreBinaryIntrinsic::StringEndsWith => member_call(
                left,
                "endsWith",
                vec![right],
                result,
                JavaMemberOrigin::Known(JavaKnownMethod::StringEndsWith),
            ),
            CoreBinaryIntrinsic::StringTruncateUtf8Bytes => runtime_call(
                JavaRuntimeCallable::StringTruncateUtf8Bytes,
                vec![left, right],
                result,
            ),
            CoreBinaryIntrinsic::StringTrimStart => runtime_call(
                JavaRuntimeCallable::StringTrimStart,
                vec![left, right],
                result,
            ),
            CoreBinaryIntrinsic::StringTrimEnd => runtime_call(
                JavaRuntimeCallable::StringTrimEnd,
                vec![left, right],
                result,
            ),
            CoreBinaryIntrinsic::BytesConcat => {
                runtime_call(JavaRuntimeCallable::BytesConcat, vec![left, right], result)
            }
            CoreBinaryIntrinsic::ListGetChecked => {
                return Ok(runtime_fallible(
                    JavaRuntimeCallable::ListGet,
                    vec![left, right],
                    result,
                ));
            }
            CoreBinaryIntrinsic::ListAppend => {
                runtime_call(JavaRuntimeCallable::ListAppend, vec![left, right], result)
            }
            CoreBinaryIntrinsic::ListConcat => {
                runtime_call(JavaRuntimeCallable::ListConcat, vec![left, right], result)
            }
            CoreBinaryIntrinsic::ListContains => {
                runtime_call(JavaRuntimeCallable::ListContains, vec![left, right], result)
            }
            CoreBinaryIntrinsic::ListIndexOf => {
                runtime_call(JavaRuntimeCallable::ListIndexOf, vec![left, right], result)
            }
            CoreBinaryIntrinsic::OptionUnwrapOr => {
                let present = runtime_call(
                    JavaRuntimeCallable::OptionIsSome,
                    vec![left.clone()],
                    boolean,
                );
                let value =
                    runtime_call(JavaRuntimeCallable::OptionValue, vec![left], result.clone());
                conditional(present, value, right, result)
            }
        };
        Ok(JavaIntrinsicExpr::Direct(value))
    }

    fn expr_list(
        &self,
        values: &[CoreExprId],
        callable_return: CoreTypeId,
    ) -> Result<(Vec<JavaStmt>, Vec<JavaExpr>), Vec<Diagnostic>> {
        let mut statements = Vec::new();
        let mut expressions = Vec::with_capacity(values.len());
        for value in values {
            let plan = self.stabilize_plan(self.expr_plan(*value, callable_return)?, "argument");
            statements.extend(plan.statements);
            expressions.push(plan.value);
        }
        Ok((statements, expressions))
    }

    fn stabilize_plan(&self, mut plan: ExprPlan, prefix: &str) -> ExprPlan {
        if matches!(
            &plan.value.kind,
            JavaExprKind::Literal(_) | JavaExprKind::Value(_)
        ) {
            return plan;
        }
        let ty = plan.value.ty.clone();
        let (name, value) = self.temporary(prefix, ty.clone());
        plan.statements.push(JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty,
            name,
            value: Some(plan.value),
        });
        plan.value = value;
        plan
    }

    fn temporary(&self, prefix: &str, ty: JavaType) -> (JavaIdentifier, JavaExpr) {
        let index = self.next_temporary.get();
        self.next_temporary.set(index + 1);
        let name = JavaIdentifier::new(format!("__polyrust_{prefix}_{index}"))
            .expect("internal Java temporary identifier is valid");
        let value = JavaExpr::local(ty, name.clone());
        (name, value)
    }

    fn success_result(
        &self,
        value: JavaExpr,
        callable_return: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        Ok(runtime_call(
            JavaRuntimeCallable::Ok,
            vec![value],
            self.poly_result_type(callable_return)?,
        ))
    }

    fn propagate_call(
        &self,
        statements: Vec<JavaStmt>,
        call: JavaExpr,
        value_type: JavaType,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let (name, _) = self.temporary("callResult", call.ty.clone());
        let JavaResultPropagationPlan { statements, value } = self
            .features
            .mapping_for::<portable_build::ResultPropagation>()
            .lower(
                &mut (),
                JavaResultPropagationInput {
                    prefix: statements,
                    call,
                    result_name: name,
                    value_type,
                    callable_result_type: self.poly_result_type(callable_return)?,
                },
            )?;
        Ok(ExprPlan { statements, value })
    }

    fn short_circuit_boolean(
        &self,
        operation: CoreBinaryIntrinsic,
        left: CoreExprId,
        right: CoreExprId,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let left = self.stabilize_plan(self.expr_plan(left, callable_return)?, "intrinsicOperand");
        let (name, result) = self.temporary("booleanResult", boolean.clone());
        let right =
            self.stabilize_plan(self.expr_plan(right, callable_return)?, "intrinsicOperand");
        let right_block = JavaBlock::new(
            right
                .statements
                .into_iter()
                .chain([JavaStmt::Assign {
                    target: result.clone(),
                    value: right.value,
                }])
                .collect(),
        );
        let (condition, then_value) = match operation {
            CoreBinaryIntrinsic::BoolAnd => (
                unary(JavaUnaryOperator::Not, left.value, boolean.clone()),
                bool_literal(false),
            ),
            CoreBinaryIntrinsic::BoolOr => (left.value, bool_literal(true)),
            _ => unreachable!("short-circuit helper only accepts boolean operations"),
        };
        let mut statements = left.statements;
        statements.push(JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty: boolean,
            name,
            value: None,
        });
        statements.push(JavaStmt::If {
            condition,
            then_block: JavaBlock::new(vec![JavaStmt::Assign {
                target: result.clone(),
                value: then_value,
            }]),
            else_block: Some(right_block),
        });
        Ok(ExprPlan {
            statements,
            value: result,
        })
    }

    fn ty(&self, id: CoreTypeId) -> Result<JavaType, Vec<Diagnostic>> {
        let value = self
            .core
            .types()
            .get(id)
            .ok_or_else(|| vec![diagnostic("missing CoreIR type")])?;
        Ok(match value {
            CoreType::Unit => JavaType::known(JavaKnownType::RuntimeUnit),
            CoreType::Bool => JavaType::primitive(JavaPrimitive::Boolean),
            CoreType::I32 => JavaType::primitive(JavaPrimitive::Int),
            CoreType::I64 => JavaType::primitive(JavaPrimitive::Long),
            CoreType::F64 => JavaType::primitive(JavaPrimitive::Double),
            CoreType::Char => JavaType::known(JavaKnownType::RuntimeScalar),
            CoreType::String => JavaType::known(JavaKnownType::String),
            CoreType::Bytes => JavaType::known(JavaKnownType::RuntimeBytes),
            CoreType::List(inner) => {
                JavaType::generic(JavaKnownType::List, vec![self.ty(*inner)?.boxed()])
            }
            CoreType::Option(inner) => {
                JavaType::generic(JavaKnownType::RuntimeOption, vec![self.ty(*inner)?.boxed()])
            }
            CoreType::Result { ok, error } => JavaType::generic(
                JavaKnownType::RuntimeValueResult,
                vec![self.ty(*ok)?.boxed(), self.ty(*error)?.boxed()],
            ),
            CoreType::Record(id) => JavaType::Reference(JavaTypeName::Generated(self.records[id])),
            CoreType::Enum(id) => JavaType::Reference(JavaTypeName::Generated(self.enums[id])),
            CoreType::Interface(id) => {
                JavaType::Reference(JavaTypeName::Generated(self.interfaces[id]))
            }
        })
    }

    fn poly_result_type(&self, result: CoreTypeId) -> Result<JavaType, Vec<Diagnostic>> {
        Ok(JavaType::generic(
            JavaKnownType::RuntimeResult,
            vec![self.ty(result)?.boxed()],
        ))
    }
}

pub(crate) fn lower_intrinsic_expression(
    value: CoreIntrinsicExpr<JavaExpr>,
    result: JavaType,
) -> Result<JavaIntrinsicExpr, Vec<Diagnostic>> {
    Lowering::<'static>::intrinsic_java_raw(value, result)
}

pub(crate) fn path(value: &str) -> RelativeOutputPath {
    RelativeOutputPath::new(value).expect("static Java output path is safe")
}

pub(crate) fn source(value: &str) -> SourceRef {
    SourceRef::logical(["java-lowering", value])
}

fn diagnostic(message: &str) -> Diagnostic {
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

fn collect_constant_dependencies(
    value: &CoreConstantExpr,
    dependencies: &mut BTreeSet<CoreConstantId>,
) {
    match &value.kind {
        CoreConstantExprKind::Constant(id) => {
            dependencies.insert(*id);
        }
        CoreConstantExprKind::Record { fields, .. } | CoreConstantExprKind::Enum { fields, .. } => {
            for field in fields {
                collect_constant_dependencies(&field.value, dependencies);
            }
        }
        CoreConstantExprKind::Some(value)
        | CoreConstantExprKind::Ok { value, .. }
        | CoreConstantExprKind::Err { value, .. } => {
            collect_constant_dependencies(value, dependencies);
        }
        CoreConstantExprKind::List { elements, .. } => {
            for element in elements {
                collect_constant_dependencies(element, dependencies);
            }
        }
        CoreConstantExprKind::Intrinsic(value) => match value.as_ref() {
            CoreIntrinsicExpr::Unary { operand, .. } => {
                collect_constant_dependencies(operand, dependencies);
            }
            CoreIntrinsicExpr::Binary { left, right, .. } => {
                collect_constant_dependencies(left, dependencies);
                collect_constant_dependencies(right, dependencies);
            }
            CoreIntrinsicExpr::Ternary {
                first,
                second,
                third,
                ..
            } => {
                collect_constant_dependencies(first, dependencies);
                collect_constant_dependencies(second, dependencies);
                collect_constant_dependencies(third, dependencies);
            }
            CoreIntrinsicExpr::Variadic { arguments, .. } => {
                for argument in arguments {
                    collect_constant_dependencies(argument, dependencies);
                }
            }
        },
        CoreConstantExprKind::Literal(_) | CoreConstantExprKind::None { .. } => {}
    }
}

pub(crate) fn identifier(value: &str) -> JavaIdentifier {
    JavaIdentifier::from_portable(value)
}

pub(crate) fn visibility_modifier(value: Visibility) -> JavaModifier {
    match value {
        Visibility::Public => JavaModifier::Public,
        Visibility::Package => JavaModifier::Private,
    }
}

pub(crate) fn private_constructor(name: &str) -> JavaConstructor {
    JavaConstructor {
        modifiers: vec![JavaModifier::Private],
        name: identifier(name),
        parameters: vec![],
        body: JavaBlock::new(vec![]),
    }
}

fn public_factory_method(
    name: &str,
    return_type: JavaType,
    parameters: Vec<JavaParameter>,
    mut result: ExprPlan,
) -> JavaMember {
    result.statements.push(JavaStmt::Return(Some(result.value)));
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public, JavaModifier::Static],
        type_parameters: vec![],
        return_type,
        name: JavaIdentifier::new(name).expect("internal Java factory identifier is valid"),
        parameters,
        body: Some(JavaBlock::new(result.statements)),
    })
}

fn length_prefixed_type(category: &str, payload: &str) -> String {
    format!("{category}{}_{}", payload.len(), payload)
}

pub(crate) fn java_unit_value() -> JavaExpr {
    new_known(
        JavaKnownConstructor::RuntimeUnit,
        JavaType::known(JavaKnownType::RuntimeUnit),
        vec![],
    )
}

pub(crate) fn bool_literal(value: bool) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Boolean),
        JavaLiteral::Boolean(value),
    )
}

pub(crate) fn i32_literal(value: i32) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Int),
        JavaLiteral::I32(value),
    )
}

pub(crate) fn i64_literal(value: i64) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Long),
        JavaLiteral::I64(value),
    )
}

pub(crate) fn f64_literal(value: u64) -> JavaExpr {
    known_call(
        JavaKnownCallable::DoubleFromLongBits,
        vec![i64_literal(value as i64)],
    )
}

pub(crate) fn scalar_literal(value: char) -> JavaExpr {
    new_known(
        JavaKnownConstructor::RuntimeScalar,
        JavaType::known(JavaKnownType::RuntimeScalar),
        vec![JavaExpr::literal(
            JavaType::primitive(JavaPrimitive::Int),
            JavaLiteral::CharScalar(u32::from(value)),
        )],
    )
}

pub(crate) fn string_literal(value: &str) -> JavaExpr {
    JavaExpr::literal(
        JavaType::known(JavaKnownType::String),
        JavaLiteral::String(value.to_owned()),
    )
}

pub(crate) fn unary(operator: JavaUnaryOperator, operand: JavaExpr, ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator,
            operand: Box::new(operand),
        },
    }
}

pub(crate) fn binary(
    operator: JavaBinaryOperator,
    left: JavaExpr,
    right: JavaExpr,
    ty: JavaType,
) -> JavaExpr {
    let precedence = match operator {
        JavaBinaryOperator::LogicalOr => JavaPrecedence::LogicalOr,
        JavaBinaryOperator::LogicalAnd => JavaPrecedence::LogicalAnd,
        JavaBinaryOperator::BitOr => JavaPrecedence::BitOr,
        JavaBinaryOperator::BitXor => JavaPrecedence::BitXor,
        JavaBinaryOperator::BitAnd => JavaPrecedence::BitAnd,
        JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual => JavaPrecedence::Equality,
        JavaBinaryOperator::Less
        | JavaBinaryOperator::LessEqual
        | JavaBinaryOperator::Greater
        | JavaBinaryOperator::GreaterEqual => JavaPrecedence::Relational,
        JavaBinaryOperator::ShiftLeft | JavaBinaryOperator::ShiftRight => JavaPrecedence::Shift,
        JavaBinaryOperator::Add | JavaBinaryOperator::Subtract => JavaPrecedence::Additive,
        JavaBinaryOperator::Multiply
        | JavaBinaryOperator::Divide
        | JavaBinaryOperator::Remainder => JavaPrecedence::Multiplicative,
    };
    JavaExpr {
        ty,
        precedence,
        kind: JavaExprKind::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        },
    }
}

fn conditional(
    condition: JavaExpr,
    when_true: JavaExpr,
    when_false: JavaExpr,
    ty: JavaType,
) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(condition),
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
        },
    }
}

fn instance_of(value: JavaExpr, target: JavaType, binding: Option<JavaIdentifier>) -> JavaExpr {
    JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Boolean),
        precedence: JavaPrecedence::Relational,
        kind: JavaExprKind::InstanceOf {
            value: Box::new(value),
            target,
            binding,
        },
    }
}

fn known_call(callable: JavaKnownCallable, arguments: Vec<JavaExpr>) -> JavaExpr {
    let signature = callable.signature();
    JavaExpr {
        ty: signature.result.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable,
                signature,
            },
            receiver: None,
            arguments,
        },
    }
}

pub(crate) fn known_generic_call(
    callable: JavaKnownCallable,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaExpr {
    let signature = JavaMethodSignature {
        receiver: None,
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable,
                signature,
            },
            receiver: None,
            arguments,
        },
    }
}

pub(crate) fn runtime_call(
    callable: JavaRuntimeCallable,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaExpr {
    let signature = JavaMethodSignature {
        receiver: None,
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Runtime {
                callable,
                signature,
            },
            receiver: None,
            arguments,
        },
    }
}

fn runtime_fallible(
    callable: JavaRuntimeCallable,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaIntrinsicExpr {
    let wrapped = JavaType::generic(JavaKnownType::RuntimeResult, vec![result.clone().boxed()]);
    let call = runtime_call(callable, arguments, wrapped);
    JavaIntrinsicExpr::Fallible {
        call,
        value_type: result,
    }
}

pub(crate) fn member_call(
    receiver: JavaExpr,
    name: &str,
    arguments: Vec<JavaExpr>,
    result: JavaType,
    origin: JavaMemberOrigin,
) -> JavaExpr {
    let signature = JavaMethodSignature {
        receiver: Some(receiver.ty.clone()),
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: receiver.ty.clone(),
                name: identifier(name),
                signature,
                origin,
            },
            receiver: Some(Box::new(receiver)),
            arguments,
        },
    }
}

fn new_known(
    constructor: JavaKnownConstructor,
    owner: JavaType,
    arguments: Vec<JavaExpr>,
) -> JavaExpr {
    JavaExpr {
        ty: owner.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::New {
            constructor: JavaConstructorRef::Known {
                constructor,
                owner,
                parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
            },
            arguments,
        },
    }
}
