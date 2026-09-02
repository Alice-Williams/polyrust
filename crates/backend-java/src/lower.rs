use std::collections::BTreeMap;

use portable_codegen::{
    BackendOptions, FileGroupRole, GeneratedCallable, GeneratedCallableId,
    GeneratedInterfaceMethod, GeneratedInterfaceMethodId, GeneratedOrigin, GeneratedSymbolId,
    GeneratedType, GeneratedTypeId, GeneratedValue, GeneratedValueId, RelativeOutputPath,
    SourceRole, SynthesisReason, TargetArtifact, TargetAstBuilder, TargetAstPackage,
    TargetCallableSignature, TargetFile, TargetFileGroup, TargetFileMember, TargetLowerer,
    VerifiedCore,
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

use crate::{ast::*, dialect::*};

#[derive(Clone, Copy, Debug, Default)]
pub struct JavaLowerer;

impl TargetLowerer<CoreProgram, JavaDialect> for JavaLowerer {
    fn lower_target(
        &self,
        core: &VerifiedCore<CoreProgram>,
        _options: &BackendOptions,
    ) -> Result<TargetAstPackage<JavaDialect>, Vec<Diagnostic>> {
        Lowering::new(core.value()).lower()
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
    interfaces: BTreeMap<CoreInterfaceId, GeneratedTypeId>,
    functions: BTreeMap<CoreFunctionId, GeneratedCallableId>,
    interface_methods: BTreeMap<CoreInterfaceMethodId, GeneratedInterfaceMethodId>,
    constants: BTreeMap<CoreConstantId, GeneratedValueId>,
}

impl<'a> Lowering<'a> {
    fn new(core: &'a CoreProgram) -> Self {
        Self {
            core,
            builder: TargetAstBuilder::new(JavaDialect),
            declared: vec![],
            entry: None,
            records: BTreeMap::new(),
            enums: BTreeMap::new(),
            variants: BTreeMap::new(),
            interfaces: BTreeMap::new(),
            functions: BTreeMap::new(),
            interface_methods: BTreeMap::new(),
            constants: BTreeMap::new(),
        }
    }

    fn lower(mut self) -> Result<TargetAstPackage<JavaDialect>, Vec<Diagnostic>> {
        self.register_types();
        self.register_values_and_callables()?;
        let negative = self.negative_file()?;
        let generated = self.generated_file()?;
        let runtime = self.runtime_file()?;
        let conformance = self.conformance_file()?;
        let native_test = self.native_test_file()?;
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
                    let generated = self.builder.generated_type(GeneratedType {
                        name: item.header.name.clone(),
                        kind: JavaDeclarationKind::SealedInterface,
                        visibility: java_visibility(item.header.visibility),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: item.header.source.clone(),
                    });
                    self.enums.insert(id, generated);
                    self.declared.push(GeneratedSymbolId::Type(generated));
                    for variant in &item.variants {
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
                        kind: JavaDeclarationKind::Interface,
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
                        ty: JavaDialect.coarse_type(&java_type),
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
                            self.ty(parameter.ty).map(|ty| JavaDialect.coarse_type(&ty))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = self.poly_result_type(value.return_type)?;
                    let symbol = self.builder.callable(GeneratedCallable {
                        name: value.header.name.clone(),
                        signature: TargetCallableSignature {
                            invocation: JavaInvocationKind::Static,
                            receiver: None,
                            parameters,
                            return_type: JavaDialect.coarse_type(&result),
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
                                self.ty(parameter.ty).map(|ty| JavaDialect.coarse_type(&ty))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let receiver = JavaType::Reference(JavaTypeName::Generated(owner));
                        let result = self.poly_result_type(method.return_type)?;
                        let symbol = self.builder.interface_method(GeneratedInterfaceMethod {
                            owner,
                            name: method.header.name.clone(),
                            signature: TargetCallableSignature {
                                invocation: JavaInvocationKind::Instance,
                                receiver: Some(JavaDialect.coarse_type(&receiver)),
                                parameters,
                                return_type: JavaDialect.coarse_type(&result),
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
                | CoreDeclaration::Enum(_)
                | CoreDeclaration::Implementation(_)
                | CoreDeclaration::Test(_) => {}
            }
        }
        Ok(())
    }

    fn generated_file(&mut self) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let mut members = Vec::new();
        for declaration in &self.core.module().declarations {
            match *declaration {
                CoreDeclaration::Constant(id) => {
                    members.push(JavaMember::Field(self.constant_field(id)?))
                }
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
                CoreDeclaration::Alias(_)
                | CoreDeclaration::Implementation(_)
                | CoreDeclaration::Test(_) => {}
            }
        }
        members.insert(0, JavaMember::Constructor(private_constructor("Generated")));
        let declaration = JavaTypeDeclaration {
            declared: self.entry,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            modifiers: vec![],
            name: identifier("Generated"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members,
        };
        Ok(self.builder.file(TargetFile::new(
            path("src/main/java/org/polyrust/generated/Generated.java"),
            SourceRole::PublicApi,
            JavaPackage::Generated,
            JavaFilePlacement::Main,
            vec![JavaFileItem::Type {
                declared: self.declared.clone(),
                declaration,
            }],
            JavaTemplateId::CompilationUnit,
            source("generated-file"),
        )))
    }

    fn runtime_file(&mut self) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let declaration = JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            modifiers: vec![],
            name: identifier("Runtime"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::Constructor(private_constructor("Runtime"))],
        };
        Ok(self.builder.file(TargetFile::new(
            path("src/main/java/org/polyrust/generated/Runtime.java"),
            SourceRole::Runtime,
            JavaPackage::Generated,
            JavaFilePlacement::Runtime,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaTemplateId::CompilationUnit,
            source("runtime-file"),
        )))
    }

    fn conformance_file(&mut self) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let int = JavaType::primitive(JavaPrimitive::Int);
        let long = JavaType::primitive(JavaPrimitive::Long);
        let string = JavaType::known(JavaKnownType::String);
        let result_long = JavaType::generic(
            JavaKnownType::RuntimeResult,
            vec![JavaType::Boxed(JavaPrimitive::Long)],
        );
        let result_int = JavaType::generic(
            JavaKnownType::RuntimeResult,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        );
        let integer_list = JavaType::generic(
            JavaKnownType::List,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        );
        let astral = identifier("astral");
        let invalid = identifier("invalid");
        let overflow = identifier("overflow");
        let original = identifier("original");
        let appended = identifier("appended");
        let statements = vec![
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: result_long.clone(),
                name: astral.clone(),
                value: Some(runtime_call(
                    JavaRuntimeCallable::ScalarLength,
                    vec![string_literal("😀")],
                    result_long.clone(),
                )),
            },
            assert_true(
                binary(
                    JavaBinaryOperator::LogicalAnd,
                    member_call(
                        JavaExpr::local(result_long.clone(), astral.clone()),
                        "ok",
                        vec![],
                        boolean.clone(),
                        JavaMemberOrigin::GeneratedDelegation,
                    ),
                    binary(
                        JavaBinaryOperator::Equal,
                        member_call(
                            JavaExpr::local(result_long.clone(), astral),
                            "value",
                            vec![],
                            long.clone(),
                            JavaMemberOrigin::GeneratedDelegation,
                        ),
                        i64_literal(1),
                        boolean.clone(),
                    ),
                    boolean.clone(),
                ),
                "astral scalar length",
            ),
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: result_long.clone(),
                name: invalid.clone(),
                value: Some(runtime_call(
                    JavaRuntimeCallable::ScalarLength,
                    vec![JavaExpr::literal(
                        string,
                        JavaLiteral::Utf16Units(vec![0xd800]),
                    )],
                    result_long.clone(),
                )),
            },
            assert_true(
                unary(
                    JavaUnaryOperator::Not,
                    member_call(
                        JavaExpr::local(result_long, invalid),
                        "ok",
                        vec![],
                        boolean.clone(),
                        JavaMemberOrigin::GeneratedDelegation,
                    ),
                    boolean.clone(),
                ),
                "unpaired surrogate rejection",
            ),
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: result_int.clone(),
                name: overflow.clone(),
                value: Some(runtime_call(
                    JavaRuntimeCallable::CheckedAddI32,
                    vec![i32_literal(i32::MAX), i32_literal(1)],
                    result_int.clone(),
                )),
            },
            assert_true(
                unary(
                    JavaUnaryOperator::Not,
                    member_call(
                        JavaExpr::local(result_int, overflow),
                        "ok",
                        vec![],
                        boolean.clone(),
                        JavaMemberOrigin::GeneratedDelegation,
                    ),
                    boolean.clone(),
                ),
                "checked overflow",
            ),
            assert_true(
                runtime_call(
                    JavaRuntimeCallable::FloatIsNegativeZero,
                    vec![f64_literal(0x8000_0000_0000_0000)],
                    boolean.clone(),
                ),
                "negative zero bits",
            ),
            assert_true(
                unary(
                    JavaUnaryOperator::Not,
                    runtime_call(
                        JavaRuntimeCallable::SemanticEqual,
                        vec![
                            f64_literal(0x7ff8_0000_0000_0001),
                            f64_literal(0x7ff8_0000_0000_0001),
                        ],
                        boolean.clone(),
                    ),
                    boolean.clone(),
                ),
                "NaN semantic inequality",
            ),
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: integer_list.clone(),
                name: original.clone(),
                value: Some(known_generic_call(
                    JavaKnownCallable::ListOf,
                    vec![i32_literal(1)],
                    integer_list.clone(),
                )),
            },
            JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: integer_list.clone(),
                name: appended.clone(),
                value: Some(runtime_call(
                    JavaRuntimeCallable::ListAppend,
                    vec![
                        JavaExpr::local(integer_list.clone(), original.clone()),
                        i32_literal(2),
                    ],
                    integer_list.clone(),
                )),
            },
            assert_true(
                binary(
                    JavaBinaryOperator::Equal,
                    member_call(
                        JavaExpr::local(integer_list.clone(), appended),
                        "size",
                        vec![],
                        int.clone(),
                        JavaMemberOrigin::Known(JavaKnownMethod::ListSize),
                    ),
                    i32_literal(2),
                    boolean.clone(),
                ),
                "append result",
            ),
            assert_true(
                binary(
                    JavaBinaryOperator::Equal,
                    member_call(
                        JavaExpr::local(integer_list, original),
                        "size",
                        vec![],
                        int,
                        JavaMemberOrigin::Known(JavaKnownMethod::ListSize),
                    ),
                    i32_literal(1),
                    boolean,
                ),
                "immutable list input",
            ),
        ];
        let declaration = JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            modifiers: vec![],
            name: identifier("ConformanceTest"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![
                JavaMember::Constructor(private_constructor("ConformanceTest")),
                JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Structural,
                    annotations: vec![],
                    modifiers: vec![JavaModifier::Public, JavaModifier::Static],
                    type_parameters: vec![],
                    return_type: JavaType::primitive(JavaPrimitive::Void),
                    name: identifier("main"),
                    parameters: vec![JavaParameter {
                        ty: JavaType::Array {
                            component: Box::new(JavaType::known(JavaKnownType::String)),
                            ownership: JavaArrayOwnership::DefensiveCopyBoundary,
                        },
                        name: identifier("arguments"),
                        final_parameter: true,
                    }],
                    body: Some(JavaBlock::new(statements)),
                }),
            ],
        };
        Ok(self.builder.file(TargetFile::new(
            path("src/test/java/org/polyrust/generated/ConformanceTest.java"),
            SourceRole::Conformance,
            JavaPackage::Generated,
            JavaFilePlacement::Conformance,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaTemplateId::CompilationUnit,
            source("conformance-file"),
        )))
    }

    fn negative_file(&mut self) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let option_integer = JavaType::generic(
            JavaKnownType::RuntimeOption,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        );
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
                    expected_type: option_integer,
                    name: identifier("invalid"),
                    initializer: string_literal("missing"),
                }),
            ],
        };
        Ok(self.builder.file(TargetFile::new(
            path("negative/InvalidTypes.java"),
            SourceRole::NegativeTest,
            JavaPackage::Generated,
            JavaFilePlacement::NegativeTest,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaTemplateId::CompilationUnit,
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
            JavaMember::Method(self.semantic_method(self.records[&id], &record.fields)?),
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
        Ok(JavaTypeDeclaration {
            declared: Some(self.records[&id]),
            kind: JavaDeclarationKind::Record,
            visibility: java_visibility(record.header.visibility),
            modifiers: vec![JavaModifier::Static],
            name: identifier(&record.header.name),
            type_parameters: vec![],
            record_components: record
                .fields
                .iter()
                .map(|field| {
                    let value = self.core.field(*field).expect("verified field");
                    Ok(JavaRecordComponent {
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
            permits: vec![],
            members,
        })
    }

    fn enum_declarations(
        &self,
        id: CoreEnumId,
    ) -> Result<Vec<JavaTypeDeclaration>, Vec<Diagnostic>> {
        let enumeration = self.core.enumeration(id).expect("verified enum");
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
                    JavaMember::Method(self.semantic_method(variant_type, &variant.fields)?),
                ],
            });
        }
        Ok(output)
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
            let value = match &ty {
                JavaType::Primitive(_) | JavaType::Boxed(_) => input,
                JavaType::Generic {
                    raw: JavaTypeName::Known(JavaKnownType::List),
                    ..
                } => known_generic_call(JavaKnownCallable::ListCopyOf, vec![input], ty.clone()),
                JavaType::Reference(_)
                | JavaType::Array { .. }
                | JavaType::Generic { .. }
                | JavaType::Wildcard { .. }
                | JavaType::TypeVariable(_) => known_generic_call(
                    JavaKnownCallable::ObjectsRequireNonNull,
                    vec![input],
                    ty.clone(),
                ),
            };
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
                value,
            });
        }
        Ok(JavaConstructor {
            modifiers: vec![visibility_modifier(visibility)],
            name: identifier(name),
            parameters,
            body: JavaBlock::new(statements),
        })
    }

    fn semantic_method(
        &self,
        owner: GeneratedTypeId,
        fields: &[CoreFieldId],
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
                    JavaRuntimeCallable::SemanticEqual,
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
            name: identifier("semanticEquals"),
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
        let mut members = Vec::new();
        for method_id in &interface.methods {
            let method = self
                .core
                .interface_method(*method_id)
                .expect("verified method");
            members.push(JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Interface(self.interface_methods[method_id]),
                annotations: vec![],
                modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
                type_parameters: vec![],
                return_type: self.poly_result_type(method.return_type)?,
                name: identifier(&method.header.name),
                parameters: self.parameters(&method.parameters)?,
                body: None,
            }));
        }
        Ok(JavaTypeDeclaration {
            declared: Some(self.interfaces[&id]),
            kind: JavaDeclarationKind::Interface,
            visibility: java_visibility(interface.header.visibility),
            modifiers: vec![JavaModifier::Static],
            name: identifier(&interface.header.name),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members,
        })
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
        Ok(JavaField {
            declared: Some(self.constants[&id]),
            modifiers: vec![
                visibility_modifier(value.header.visibility),
                JavaModifier::Static,
                JavaModifier::Final,
            ],
            ty: self.ty(value.ty)?,
            name: identifier(&value.header.name),
            initializer: Some(self.constant_expr(&value.value, value.ty)?),
        })
    }

    fn function_method(&self, id: CoreFunctionId) -> Result<JavaMethod, Vec<Diagnostic>> {
        let value = self.core.function(id).expect("verified function");
        Ok(JavaMethod {
            declared: JavaMethodDeclaration::Callable(self.functions[&id]),
            annotations: vec![],
            modifiers: vec![
                visibility_modifier(value.header.visibility),
                JavaModifier::Static,
            ],
            type_parameters: vec![],
            return_type: self.poly_result_type(value.return_type)?,
            name: identifier(&value.header.name),
            parameters: self.parameters(&value.parameters)?,
            body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
                self.capture_block(value.body, value.return_type)?,
            ))])),
        })
    }

    fn implementation_method(
        &self,
        id: CoreImplementationMethodId,
    ) -> Result<JavaMethod, Vec<Diagnostic>> {
        let value = self
            .core
            .implementation_method(id)
            .expect("verified implementation method");
        Ok(JavaMethod {
            declared: JavaMethodDeclaration::Implementation {
                method: id,
                interface: self.interface_methods[&value.interface_method],
            },
            annotations: vec![JavaAnnotation::Override],
            modifiers: vec![JavaModifier::Public],
            type_parameters: vec![],
            return_type: self.poly_result_type(value.return_type)?,
            name: identifier(&value.header.name),
            parameters: self.parameters(&value.parameters)?,
            body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
                self.capture_block(value.body, value.return_type)?,
            ))])),
        })
    }

    fn native_test_file(&mut self) -> Result<portable_codegen::TargetFileId, Vec<Diagnostic>> {
        let declaration = self.native_test_declaration()?;
        Ok(self.builder.file(TargetFile::new(
            path("src/test/java/org/polyrust/generated/GeneratedTest.java"),
            SourceRole::NativeTest,
            JavaPackage::Generated,
            JavaFilePlacement::NativeTest,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaTemplateId::CompilationUnit,
            source("native-test-file"),
        )))
    }

    fn capture_block(
        &self,
        block: CoreBlockId,
        result: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let value_type = self.ty(result)?;
        let action_type = JavaType::generic(
            JavaKnownType::RuntimeAction,
            vec![value_type.clone().boxed()],
        );
        let result_type = self.poly_result_type(result)?;
        Ok(runtime_call(
            JavaRuntimeCallable::Capture,
            vec![JavaExpr {
                ty: action_type,
                precedence: JavaPrecedence::Assignment,
                kind: JavaExprKind::Lambda {
                    parameters: vec![],
                    body: self.block(block, BlockMode::ReturnResult)?,
                },
            }],
            result_type,
        ))
    }

    fn evaluate_block(
        &self,
        block: CoreBlockId,
        result: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let value_type = self.ty(result)?;
        let action_type = JavaType::generic(
            JavaKnownType::RuntimeAction,
            vec![value_type.clone().boxed()],
        );
        Ok(runtime_call(
            JavaRuntimeCallable::Evaluate,
            vec![JavaExpr {
                ty: action_type,
                precedence: JavaPrecedence::Assignment,
                kind: JavaExprKind::Lambda {
                    parameters: vec![],
                    body: self.block(block, BlockMode::ReturnResult)?,
                },
            }],
            value_type,
        ))
    }

    fn block(&self, id: CoreBlockId, mode: BlockMode) -> Result<JavaBlock, Vec<Diagnostic>> {
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
                    statements.push(JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: self.ty(binding.ty)?,
                        name: identifier(&binding.name),
                        value: Some(self.expr(*value)?),
                    });
                }
                CoreStatement::ForEach {
                    binding,
                    iterable,
                    body,
                    ..
                } => {
                    let binding = self.core.local(*binding).expect("verified local");
                    statements.push(JavaStmt::ForEach {
                        binding_type: self.ty(binding.ty)?,
                        binding: identifier(&binding.name),
                        iterable: self.expr(*iterable)?,
                        body: self.block(*body, BlockMode::StatementBody)?,
                    });
                }
                CoreStatement::Return { value, .. } => {
                    statements.push(JavaStmt::Return(Some(match value {
                        Some(value) => self.expr(*value)?,
                        None => unit_value(),
                    })));
                }
                CoreStatement::Evaluate { value, .. } => {
                    statements.push(JavaStmt::Expression(self.expr(*value)?));
                }
            }
        }
        if let Some(result) = block.result {
            let value = self.expr(result)?;
            statements.push(match mode {
                BlockMode::ReturnResult => JavaStmt::Return(Some(value)),
                BlockMode::StatementBody => JavaStmt::Expression(value),
            });
        } else if mode == BlockMode::ReturnResult {
            statements.push(JavaStmt::Return(Some(unit_value())));
        }
        Ok(JavaBlock::new(statements))
    }

    fn expr(&self, id: CoreExprId) -> Result<JavaExpr, Vec<Diagnostic>> {
        let expression = self
            .core
            .expressions()
            .get(id)
            .ok_or_else(|| vec![diagnostic("missing CoreIR expression")])?;
        let ty = self.ty(expression.ty)?;
        match &expression.kind {
            CoreExprKind::Literal(value) => self.value(value, expression.ty),
            CoreExprKind::Local(id) => {
                let local_value = self.core.local(*id).expect("verified local");
                Ok(JavaExpr::local(ty, identifier(&local_value.name)))
            }
            CoreExprKind::Constant(id) => Ok(JavaExpr {
                ty,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(
                    self.constants[id],
                ))),
            }),
            CoreExprKind::SelfValue(id) => Ok(JavaExpr {
                ty: JavaType::Reference(JavaTypeName::Generated(self.records[id])),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::This),
            }),
            CoreExprKind::ConstructRecord { record, fields } => {
                self.construct_generated(self.records[record], fields, ty)
            }
            CoreExprKind::ConstructEnum {
                variant, fields, ..
            } => self.construct_generated(self.variants[variant], fields, ty),
            CoreExprKind::ConstructSome(value) => Ok(runtime_call(
                JavaRuntimeCallable::OptionSome,
                vec![self.expr(*value)?],
                ty,
            )),
            CoreExprKind::ConstructNone { .. } => {
                Ok(runtime_call(JavaRuntimeCallable::OptionNone, vec![], ty))
            }
            CoreExprKind::ConstructOk { value, .. } => Ok(runtime_call(
                JavaRuntimeCallable::ValueResultOk,
                vec![self.expr(*value)?],
                ty,
            )),
            CoreExprKind::ConstructErr { value, .. } => Ok(runtime_call(
                JavaRuntimeCallable::ValueResultErr,
                vec![self.expr(*value)?],
                ty,
            )),
            CoreExprKind::ConstructList { elements, .. } => {
                let values = elements
                    .iter()
                    .map(|value| self.expr(*value))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(known_generic_call(JavaKnownCallable::ListOf, values, ty))
            }
            CoreExprKind::CoerceInterface { value, .. } => Ok(JavaExpr {
                ty: ty.clone(),
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Cast {
                    target: ty,
                    value: Box::new(self.expr(*value)?),
                },
            }),
            CoreExprKind::Field { value, field } => {
                let field_value = self.core.field(*field).expect("verified field");
                Ok(member_call(
                    self.expr(*value)?,
                    &field_value.header.name,
                    vec![],
                    ty,
                    JavaMemberOrigin::GeneratedField(*field),
                ))
            }
            CoreExprKind::Call {
                function,
                arguments,
            } => {
                let function_value = self.core.function(*function).expect("verified function");
                let arguments = arguments
                    .iter()
                    .map(|value| self.expr(*value))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = self.poly_result_type(function_value.return_type)?;
                let signature = JavaMethodSignature {
                    receiver: None,
                    parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
                    result: result.clone(),
                    checked_exceptions: vec![],
                    nullable_result: false,
                    pure: true,
                };
                let call = JavaExpr {
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
                };
                Ok(runtime_call(JavaRuntimeCallable::Unwrap, vec![call], ty))
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
                let arguments = arguments
                    .iter()
                    .map(|value| self.expr(*value))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = self.poly_result_type(method_value.return_type)?;
                let call = member_call(
                    self.expr(*receiver)?,
                    &method_value.header.name,
                    arguments,
                    result,
                    JavaMemberOrigin::GeneratedImplementation(*method),
                );
                Ok(runtime_call(JavaRuntimeCallable::Unwrap, vec![call], ty))
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
                let receiver = self.expr(*receiver)?;
                let arguments = arguments
                    .iter()
                    .map(|value| self.expr(*value))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = self.poly_result_type(method_value.return_type)?;
                let signature = JavaMethodSignature {
                    receiver: Some(receiver.ty.clone()),
                    parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
                    result: result.clone(),
                    checked_exceptions: vec![],
                    nullable_result: false,
                    pure: true,
                };
                let call = JavaExpr {
                    ty: result,
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Call {
                        callable: JavaCallableRef::Interface {
                            symbol: self.interface_methods[method],
                            signature,
                        },
                        receiver: Some(Box::new(receiver)),
                        arguments,
                    },
                };
                Ok(runtime_call(JavaRuntimeCallable::Unwrap, vec![call], ty))
            }
            CoreExprKind::Intrinsic(value) => self.intrinsic(value, expression.ty),
            CoreExprKind::If {
                condition,
                then_block,
                else_block,
            } => {
                let action_type =
                    JavaType::generic(JavaKnownType::RuntimeAction, vec![ty.clone().boxed()]);
                Ok(runtime_call(
                    JavaRuntimeCallable::Evaluate,
                    vec![JavaExpr {
                        ty: action_type,
                        precedence: JavaPrecedence::Assignment,
                        kind: JavaExprKind::Lambda {
                            parameters: vec![],
                            body: JavaBlock::new(vec![JavaStmt::If {
                                condition: self.expr(*condition)?,
                                then_block: self.block(*then_block, BlockMode::ReturnResult)?,
                                else_block: Some(self.block(*else_block, BlockMode::ReturnResult)?),
                            }]),
                        },
                    }],
                    ty,
                ))
            }
            CoreExprKind::Match { value, arms } => self.match_expr(*value, arms, expression.ty),
            CoreExprKind::Block(block) => self.evaluate_block(*block, expression.ty),
        }
    }

    fn construct_generated(
        &self,
        owner: GeneratedTypeId,
        fields: &[CoreExprField],
        result_type: JavaType,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let arguments = fields
            .iter()
            .map(|field| self.expr(field.value))
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

    fn value(&self, value: &CoreValue, expected: CoreTypeId) -> Result<JavaExpr, Vec<Diagnostic>> {
        let ty = self.ty(expected)?;
        Ok(match value {
            CoreValue::Unit => unit_value(),
            CoreValue::Bool(value) => bool_literal(*value),
            CoreValue::I32(value) => i32_literal(*value),
            CoreValue::I64(value) => i64_literal(*value),
            CoreValue::F64(value) => f64_literal(value.0),
            CoreValue::Char(value) => JavaExpr::literal(
                JavaType::known(JavaKnownType::String),
                JavaLiteral::CharScalar(u32::from(*value)),
            ),
            CoreValue::String(value) => string_literal(value),
            CoreValue::Bytes(values) => {
                let list = JavaType::generic(
                    JavaKnownType::List,
                    vec![JavaType::Boxed(JavaPrimitive::Int)],
                );
                let elements = values
                    .iter()
                    .map(|value| i32_literal(i32::from(*value)))
                    .collect();
                runtime_call(
                    JavaRuntimeCallable::BytesOf,
                    vec![known_generic_call(
                        JavaKnownCallable::ListOf,
                        elements,
                        list,
                    )],
                    ty,
                )
            }
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
                known_generic_call(JavaKnownCallable::ListOf, elements, ty)
            }
            CoreValue::None => runtime_call(JavaRuntimeCallable::OptionNone, vec![], ty),
            CoreValue::Some(value) => {
                let CoreType::Option(inner) = self
                    .core
                    .types()
                    .get(expected)
                    .expect("verified option type")
                else {
                    return Err(vec![diagnostic("some value does not have an option type")]);
                };
                runtime_call(
                    JavaRuntimeCallable::OptionSome,
                    vec![self.value(value, *inner)?],
                    ty,
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
                runtime_call(
                    JavaRuntimeCallable::ValueResultOk,
                    vec![self.value(value, *ok)?],
                    ty,
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
                runtime_call(
                    JavaRuntimeCallable::ValueResultErr,
                    vec![self.value(value, *error)?],
                    ty,
                )
            }
            CoreValue::Record { record, fields } => {
                self.construct_value(self.records[record], fields, ty)?
            }
            CoreValue::Enum {
                variant, fields, ..
            } => self.construct_value(self.variants[variant], fields, ty)?,
        })
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
            CoreConstantExprKind::Constant(id) => Ok(JavaExpr {
                ty,
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(
                    self.constants[id],
                ))),
            }),
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
        let owner_type = JavaType::Reference(JavaTypeName::Generated(owner));
        let arguments = fields.to_vec();
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

    fn native_test_declaration(&self) -> Result<JavaTypeDeclaration, Vec<Diagnostic>> {
        let mut statements = Vec::new();
        for (index, test) in self.core.tests().iter().enumerate() {
            let (actual, result_type) = match &test.invocation {
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
                        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
                        result: result.clone(),
                        checked_exceptions: vec![],
                        nullable_result: false,
                        pure: true,
                    };
                    (
                        JavaExpr {
                            ty: result.clone(),
                            precedence: JavaPrecedence::Primary,
                            kind: JavaExprKind::Call {
                                callable: JavaCallableRef::Generated {
                                    symbol: self.functions[function],
                                    signature,
                                },
                                receiver: None,
                                arguments,
                            },
                        },
                        result,
                    )
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
                    let receiver = self.typed_value(receiver)?;
                    let arguments = arguments
                        .iter()
                        .map(|value| self.typed_value(value))
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = self.poly_result_type(method_value.return_type)?;
                    (
                        member_call(
                            receiver,
                            &method_value.header.name,
                            arguments,
                            result.clone(),
                            JavaMemberOrigin::GeneratedImplementation(*method),
                        ),
                        result,
                    )
                }
            };
            let name = identifier(&format!("actual{index}"));
            statements.push(JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: result_type.clone(),
                name: name.clone(),
                value: Some(actual),
            });
            let actual_local = JavaExpr::local(result_type.clone(), name);
            let ok = member_call(
                actual_local.clone(),
                "ok",
                vec![],
                JavaType::primitive(JavaPrimitive::Boolean),
                JavaMemberOrigin::GeneratedDelegation,
            );
            match &test.expected {
                portable_core_ir::CoreExpectedOutcome::Value(expected) => {
                    statements.push(JavaStmt::If {
                        condition: unary(
                            JavaUnaryOperator::Not,
                            ok,
                            JavaType::primitive(JavaPrimitive::Boolean),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::ThrowAssertion(string_literal(
                            &format!("portable test {index} unexpectedly failed"),
                        ))]),
                        else_block: None,
                    });
                    let expected_value = self.typed_value(expected)?;
                    let actual_value = member_call(
                        actual_local,
                        "value",
                        vec![],
                        expected_value.ty.clone(),
                        JavaMemberOrigin::GeneratedDelegation,
                    );
                    let equal = runtime_call(
                        JavaRuntimeCallable::DeepEqual,
                        vec![actual_value, expected_value],
                        JavaType::primitive(JavaPrimitive::Boolean),
                    );
                    statements.push(JavaStmt::If {
                        condition: unary(
                            JavaUnaryOperator::Not,
                            equal,
                            JavaType::primitive(JavaPrimitive::Boolean),
                        ),
                        then_block: JavaBlock::new(vec![JavaStmt::ThrowAssertion(string_literal(
                            &format!("portable test {index} value mismatch"),
                        ))]),
                        else_block: None,
                    });
                }
                portable_core_ir::CoreExpectedOutcome::Error(_) => statements.push(JavaStmt::If {
                    condition: ok,
                    then_block: JavaBlock::new(vec![JavaStmt::ThrowAssertion(string_literal(
                        &format!("portable test {index} unexpectedly succeeded"),
                    ))]),
                    else_block: None,
                }),
            }
        }
        Ok(JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            modifiers: vec![],
            name: identifier("GeneratedTest"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![
                JavaMember::Constructor(private_constructor("GeneratedTest")),
                JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Structural,
                    annotations: vec![],
                    modifiers: vec![JavaModifier::Public, JavaModifier::Static],
                    type_parameters: vec![],
                    return_type: JavaType::primitive(JavaPrimitive::Void),
                    name: identifier("main"),
                    parameters: vec![JavaParameter {
                        ty: JavaType::Array {
                            component: Box::new(JavaType::known(JavaKnownType::String)),
                            ownership: JavaArrayOwnership::DefensiveCopyBoundary,
                        },
                        name: identifier("arguments"),
                        final_parameter: true,
                    }],
                    body: Some(JavaBlock::new(statements)),
                }),
            ],
        })
    }

    fn typed_value(&self, value: &CoreTypedValue) -> Result<JavaExpr, Vec<Diagnostic>> {
        self.value(&value.value, value.ty)
    }

    fn match_expr(
        &self,
        value: CoreExprId,
        arms: &[CoreMatchArm],
        result: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let matched = self.expr(value)?;
        let matched_type = matched.ty.clone();
        let result_type = self.ty(result)?;
        let matched_name = identifier("matchValue");
        let matched_local = JavaExpr::local(matched_type.clone(), matched_name.clone());
        let mut statements = vec![JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: matched_type,
            name: matched_name,
            value: Some(matched),
        }];
        for (index, arm) in arms.iter().enumerate() {
            let (condition, mut bindings) =
                self.pattern(&arm.pattern, matched_local.clone(), index)?;
            let mut body = self.block(arm.body, BlockMode::ReturnResult)?;
            bindings.append(&mut body.statements);
            statements.push(JavaStmt::If {
                condition,
                then_block: JavaBlock::new(bindings),
                else_block: None,
            });
        }
        statements.push(JavaStmt::ThrowAssertion(string_literal(
            "verified CoreIR match was unexpectedly non-exhaustive",
        )));
        let action_type = JavaType::generic(
            JavaKnownType::RuntimeAction,
            vec![result_type.clone().boxed()],
        );
        Ok(runtime_call(
            JavaRuntimeCallable::Evaluate,
            vec![JavaExpr {
                ty: action_type,
                precedence: JavaPrecedence::Assignment,
                kind: JavaExprKind::Lambda {
                    parameters: vec![],
                    body: JavaBlock::new(statements),
                },
            }],
            result_type,
        ))
    }

    fn pattern(
        &self,
        pattern: &CorePattern,
        matched: JavaExpr,
        index: usize,
    ) -> Result<(JavaExpr, Vec<JavaStmt>), Vec<Diagnostic>> {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        Ok(match pattern {
            CorePattern::Wildcard { .. } => (bool_literal(true), vec![]),
            CorePattern::Bool { value, .. } => (
                binary(
                    JavaBinaryOperator::Equal,
                    matched,
                    bool_literal(*value),
                    boolean,
                ),
                vec![],
            ),
            CorePattern::EnumVariant {
                variant, bindings, ..
            } => {
                let variant_type =
                    JavaType::Reference(JavaTypeName::Generated(self.variants[variant]));
                let variant_name = identifier(&format!("matchedVariant{index}"));
                let condition = JavaExpr {
                    ty: boolean,
                    precedence: JavaPrecedence::Relational,
                    kind: JavaExprKind::InstanceOf {
                        value: Box::new(matched),
                        target: variant_type.clone(),
                        binding: Some(variant_name.clone()),
                    },
                };
                let receiver = JavaExpr::local(variant_type, variant_name);
                let statements = bindings
                    .iter()
                    .map(|binding| {
                        let local_value = self
                            .core
                            .local(binding.binding)
                            .expect("verified pattern local");
                        let field = self.core.field(binding.field).expect("verified field");
                        Ok(JavaStmt::Local {
                            finality: JavaLocalFinality::Final,
                            ty: self.ty(local_value.ty)?,
                            name: identifier(&local_value.name),
                            value: Some(member_call(
                                receiver.clone(),
                                &field.header.name,
                                vec![],
                                self.ty(field.ty)?,
                                JavaMemberOrigin::GeneratedField(binding.field),
                            )),
                        })
                    })
                    .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
                (condition, statements)
            }
            CorePattern::None { .. } => {
                let some = runtime_call(
                    JavaRuntimeCallable::OptionIsSome,
                    vec![matched],
                    boolean.clone(),
                );
                (unary(JavaUnaryOperator::Not, some, boolean), vec![])
            }
            CorePattern::Some { binding, .. } => {
                let local_value = self.core.local(*binding).expect("verified pattern local");
                let condition = runtime_call(
                    JavaRuntimeCallable::OptionIsSome,
                    vec![matched.clone()],
                    boolean,
                );
                let value = runtime_call(
                    JavaRuntimeCallable::OptionValue,
                    vec![matched],
                    self.ty(local_value.ty)?,
                );
                (
                    condition,
                    vec![JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: self.ty(local_value.ty)?,
                        name: identifier(&local_value.name),
                        value: Some(value),
                    }],
                )
            }
            CorePattern::Ok { binding, .. } | CorePattern::Err { binding, .. } => {
                let local_value = self.core.local(*binding).expect("verified pattern local");
                let is_ok = runtime_call(
                    JavaRuntimeCallable::ValueResultIsOk,
                    vec![matched.clone()],
                    boolean.clone(),
                );
                let success = matches!(pattern, CorePattern::Ok { .. });
                let condition = if success {
                    is_ok
                } else {
                    unary(JavaUnaryOperator::Not, is_ok, boolean)
                };
                let callable = if success {
                    JavaRuntimeCallable::ValueResultValue
                } else {
                    JavaRuntimeCallable::ValueResultError
                };
                let value = runtime_call(callable, vec![matched], self.ty(local_value.ty)?);
                (
                    condition,
                    vec![JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: self.ty(local_value.ty)?,
                        name: identifier(&local_value.name),
                        value: Some(value),
                    }],
                )
            }
        })
    }

    fn intrinsic(
        &self,
        value: &CoreIntrinsicExpr<CoreExprId>,
        result: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let mapped = match value {
            CoreIntrinsicExpr::Unary { operation, operand } => CoreIntrinsicExpr::Unary {
                operation: *operation,
                operand: self.expr(*operand)?,
            },
            CoreIntrinsicExpr::Binary {
                operation,
                left,
                right,
            } => CoreIntrinsicExpr::Binary {
                operation: *operation,
                left: self.expr(*left)?,
                right: self.expr(*right)?,
            },
            CoreIntrinsicExpr::Ternary {
                operation,
                first,
                second,
                third,
            } => CoreIntrinsicExpr::Ternary {
                operation: *operation,
                first: self.expr(*first)?,
                second: self.expr(*second)?,
                third: self.expr(*third)?,
            },
            CoreIntrinsicExpr::Variadic {
                operation,
                arguments,
            } => CoreIntrinsicExpr::Variadic {
                operation: *operation,
                arguments: arguments
                    .iter()
                    .map(|value| self.expr(*value))
                    .collect::<Result<Vec<_>, _>>()?,
            },
        };
        self.intrinsic_java(mapped, self.ty(result)?)
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
        self.intrinsic_java(mapped, self.ty(result)?)
    }

    fn constant_untyped(
        &self,
        value: &CoreConstantExpr,
    ) -> Result<(JavaExpr, JavaType), Vec<Diagnostic>> {
        match &value.kind {
            CoreConstantExprKind::Literal(value) => self.literal_untyped(value),
            CoreConstantExprKind::Constant(id) => {
                let constant = self.core.constant(*id).expect("verified constant");
                let ty = self.ty(constant.ty)?;
                Ok((
                    JavaExpr {
                        ty: ty.clone(),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Value(JavaValueRef::Generated(
                            GeneratedSymbolId::Value(self.constants[id]),
                        )),
                    },
                    ty,
                ))
            }
            CoreConstantExprKind::Record { record, .. } => {
                let ty = JavaType::Reference(JavaTypeName::Generated(self.records[record]));
                let id = self.find_type(&CoreType::Record(*record))?;
                Ok((self.constant_expr(value, id)?, ty))
            }
            CoreConstantExprKind::Enum { enumeration, .. } => {
                let ty = JavaType::Reference(JavaTypeName::Generated(self.enums[enumeration]));
                let id = self.find_type(&CoreType::Enum(*enumeration))?;
                Ok((self.constant_expr(value, id)?, ty))
            }
            CoreConstantExprKind::Some(inner) => {
                let (inner, inner_ty) = self.constant_untyped(inner)?;
                let ty = JavaType::generic(JavaKnownType::RuntimeOption, vec![inner_ty.boxed()]);
                Ok((
                    runtime_call(JavaRuntimeCallable::OptionSome, vec![inner], ty.clone()),
                    ty,
                ))
            }
            CoreConstantExprKind::None { inner } => {
                let ty =
                    JavaType::generic(JavaKnownType::RuntimeOption, vec![self.ty(*inner)?.boxed()]);
                Ok((
                    runtime_call(JavaRuntimeCallable::OptionNone, vec![], ty.clone()),
                    ty,
                ))
            }
            CoreConstantExprKind::Ok { value, error } => {
                let (value, value_ty) = self.constant_untyped(value)?;
                let ty = JavaType::generic(
                    JavaKnownType::RuntimeValueResult,
                    vec![value_ty.boxed(), self.ty(*error)?.boxed()],
                );
                Ok((
                    runtime_call(JavaRuntimeCallable::ValueResultOk, vec![value], ty.clone()),
                    ty,
                ))
            }
            CoreConstantExprKind::Err { value, ok } => {
                let (value, value_ty) = self.constant_untyped(value)?;
                let ty = JavaType::generic(
                    JavaKnownType::RuntimeValueResult,
                    vec![self.ty(*ok)?.boxed(), value_ty.boxed()],
                );
                Ok((
                    runtime_call(JavaRuntimeCallable::ValueResultErr, vec![value], ty.clone()),
                    ty,
                ))
            }
            CoreConstantExprKind::List { element, elements } => {
                let ty = JavaType::generic(JavaKnownType::List, vec![self.ty(*element)?.boxed()]);
                let elements = elements
                    .iter()
                    .map(|value| self.constant_expr(value, *element))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((
                    known_generic_call(JavaKnownCallable::ListOf, elements, ty.clone()),
                    ty,
                ))
            }
            CoreConstantExprKind::Intrinsic(_) => Err(vec![Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                "nested constant intrinsic requires its declared result type",
                value.source.clone(),
            )]),
        }
    }

    fn literal_untyped(&self, value: &CoreValue) -> Result<(JavaExpr, JavaType), Vec<Diagnostic>> {
        Ok(match value {
            CoreValue::Unit => (unit_value(), JavaType::known(JavaKnownType::RuntimeUnit)),
            CoreValue::Bool(value) => (
                bool_literal(*value),
                JavaType::primitive(JavaPrimitive::Boolean),
            ),
            CoreValue::I32(value) => (i32_literal(*value), JavaType::primitive(JavaPrimitive::Int)),
            CoreValue::I64(value) => (
                i64_literal(*value),
                JavaType::primitive(JavaPrimitive::Long),
            ),
            CoreValue::F64(value) => (
                f64_literal(value.0),
                JavaType::primitive(JavaPrimitive::Double),
            ),
            CoreValue::Char(value) => {
                let ty = JavaType::known(JavaKnownType::String);
                (
                    JavaExpr::literal(ty.clone(), JavaLiteral::CharScalar(u32::from(*value))),
                    ty,
                )
            }
            CoreValue::String(value) => {
                let ty = JavaType::known(JavaKnownType::String);
                (string_literal(value), ty)
            }
            CoreValue::Bytes(_)
            | CoreValue::List(_)
            | CoreValue::None
            | CoreValue::Some(_)
            | CoreValue::Ok(_)
            | CoreValue::Err(_)
            | CoreValue::Record { .. }
            | CoreValue::Enum { .. } => {
                return Err(vec![diagnostic(
                    "compound literal in a nested constant intrinsic requires an explicit type",
                )]);
            }
        })
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
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        Ok(match value {
            CoreIntrinsicExpr::Unary { operation, operand } => {
                self.unary_intrinsic(operation, operand, result)?
            }
            CoreIntrinsicExpr::Binary {
                operation,
                left,
                right,
            } => self.binary_intrinsic(operation, left, right, result)?,
            CoreIntrinsicExpr::Ternary {
                operation,
                first,
                second,
                third,
            } => match operation {
                CoreTernaryIntrinsic::StringSliceScalars => runtime_call(
                    JavaRuntimeCallable::StringSliceScalars,
                    vec![first, second, third],
                    result,
                ),
                CoreTernaryIntrinsic::StringReplaceAll => member_call(
                    first,
                    "replace",
                    vec![second, third],
                    result,
                    JavaMemberOrigin::Known(JavaKnownMethod::StringReplace),
                ),
                CoreTernaryIntrinsic::BytesReplaceAll => runtime_call(
                    JavaRuntimeCallable::BytesReplaceAll,
                    vec![first, second, third],
                    result,
                ),
            },
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
                runtime_call(
                    JavaRuntimeCallable::StringReplaceMany,
                    vec![
                        source,
                        known_generic_call(JavaKnownCallable::ListOf, pairs, pair_list),
                    ],
                    result,
                )
            }
        })
    }

    fn unary_intrinsic(
        &self,
        operation: CoreUnaryIntrinsic,
        operand: JavaExpr,
        result: JavaType,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        Ok(match operation {
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
                runtime_fallible(callable, vec![operand], result)
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
                runtime_fallible(JavaRuntimeCallable::ScalarLength, vec![operand], result)
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
                runtime_fallible(JavaRuntimeCallable::NarrowI64ToI32, vec![operand], result)
            }
            CoreUnaryIntrinsic::StringToUtf8 => {
                runtime_call(JavaRuntimeCallable::StringToUtf8, vec![operand], result)
            }
            CoreUnaryIntrinsic::StringFromUtf8Checked => {
                runtime_fallible(JavaRuntimeCallable::StringFromUtf8, vec![operand], result)
            }
        })
    }

    fn binary_intrinsic(
        &self,
        operation: CoreBinaryIntrinsic,
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let numeric_width = match left.ty {
            JavaType::Primitive(JavaPrimitive::Int) => Some(false),
            JavaType::Primitive(JavaPrimitive::Long) => Some(true),
            _ => None,
        };
        let direct = |operator| binary(operator, left.clone(), right.clone(), result.clone());
        Ok(match operation {
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
            CoreBinaryIntrinsic::Less => direct(JavaBinaryOperator::Less),
            CoreBinaryIntrinsic::LessEqual => direct(JavaBinaryOperator::LessEqual),
            CoreBinaryIntrinsic::Greater => direct(JavaBinaryOperator::Greater),
            CoreBinaryIntrinsic::GreaterEqual => direct(JavaBinaryOperator::GreaterEqual),
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
                runtime_fallible(callable, vec![left, right], result)
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
                runtime_fallible(callable, vec![left, right], result)
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
                runtime_fallible(JavaRuntimeCallable::ListGet, vec![left, right], result)
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
            CoreType::Char | CoreType::String => JavaType::known(JavaKnownType::String),
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

fn path(value: &str) -> RelativeOutputPath {
    RelativeOutputPath::new(value).expect("static Java output path is safe")
}

fn source(value: &str) -> SourceRef {
    SourceRef::logical(["java-lowering", value])
}

fn diagnostic(message: &str) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::InvalidStructure, message, source("error"))
}

fn java_visibility(value: Visibility) -> JavaVisibility {
    match value {
        Visibility::Public => JavaVisibility::Public,
        Visibility::Package => JavaVisibility::Private,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockMode {
    ReturnResult,
    StatementBody,
}

fn identifier(value: &str) -> JavaIdentifier {
    JavaIdentifier::from_portable(value)
}

fn visibility_modifier(value: Visibility) -> JavaModifier {
    match value {
        Visibility::Public => JavaModifier::Public,
        Visibility::Package => JavaModifier::Private,
    }
}

fn private_constructor(name: &str) -> JavaConstructor {
    JavaConstructor {
        modifiers: vec![JavaModifier::Private],
        name: identifier(name),
        parameters: vec![],
        body: JavaBlock::new(vec![]),
    }
}

fn assert_true(condition: JavaExpr, message: &str) -> JavaStmt {
    JavaStmt::If {
        condition: unary(
            JavaUnaryOperator::Not,
            condition,
            JavaType::primitive(JavaPrimitive::Boolean),
        ),
        then_block: JavaBlock::new(vec![JavaStmt::ThrowAssertion(string_literal(message))]),
        else_block: None,
    }
}

fn unit_value() -> JavaExpr {
    new_known(
        JavaKnownConstructor::RuntimeUnit,
        JavaType::known(JavaKnownType::RuntimeUnit),
        vec![],
    )
}

fn bool_literal(value: bool) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Boolean),
        JavaLiteral::Boolean(value),
    )
}

fn i32_literal(value: i32) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Int),
        JavaLiteral::I32(value),
    )
}

fn i64_literal(value: i64) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Long),
        JavaLiteral::I64(value),
    )
}

fn f64_literal(value: u64) -> JavaExpr {
    known_call(
        JavaKnownCallable::DoubleFromLongBits,
        vec![i64_literal(value as i64)],
    )
}

fn string_literal(value: &str) -> JavaExpr {
    JavaExpr::literal(
        JavaType::known(JavaKnownType::String),
        JavaLiteral::String(value.to_owned()),
    )
}

fn unary(operator: JavaUnaryOperator, operand: JavaExpr, ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator,
            operand: Box::new(operand),
        },
    }
}

fn binary(operator: JavaBinaryOperator, left: JavaExpr, right: JavaExpr, ty: JavaType) -> JavaExpr {
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

fn known_generic_call(
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

fn runtime_call(
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
) -> JavaExpr {
    let wrapped = JavaType::generic(JavaKnownType::RuntimeResult, vec![result.clone().boxed()]);
    let call = runtime_call(callable, arguments, wrapped);
    runtime_call(JavaRuntimeCallable::Unwrap, vec![call], result)
}

fn member_call(
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
