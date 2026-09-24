use crate::ast::*;
use crate::dialect::JavaDialect;
use portable_codegen::*;
use portable_diagnostics::SourceRef;
use std::{collections::BTreeMap, sync::Arc};

pub fn source() -> SourceRef {
    SourceRef::logical(["java-source-record-test"])
}
pub fn name(value: &str) -> JavaIdentifier {
    JavaIdentifier::new(value).unwrap()
}
pub fn int() -> JavaType {
    JavaType::primitive(JavaPrimitive::Int)
}
pub fn boolean() -> JavaType {
    JavaType::primitive(JavaPrimitive::Boolean)
}
pub fn id(hash: u64) -> RustDeclarationId {
    RustDeclarationId {
        crate_id: 7,
        definition_path_hash: hash,
    }
}

pub fn origin() -> RustSourceOrigin {
    let root = Arc::new(RustModuleDocumentation {
        declaration: id(1),
        parent: None,
        location: RustSourceLocation {
            file: "src/lib.rs".into(),
            line: 1,
            column: 1,
        },
        documentation: vec![],
    });
    let ancestry: RustModuleAncestry = vec![root.clone()].into();
    RustSourceOrigin {
        declaration: id(2),
        node: RustSourceNode::Declaration,
        module: id(1),
        location: root.location.clone(),
        visibility: RustVisibility::RestrictedTo(id(1)),
        externally_reachable: false,
        documentation: vec![],
        module_ancestors: ancestry.clone(),
        crate_exports: Arc::new(RustCrateExports {
            root: id(1),
            modules: BTreeMap::from([(id(1), BTreeMap::new())]),
            module_ancestries: BTreeMap::from([(id(1), ancestry)]),
        }),
    }
}

pub struct Fixture {
    pub builder: TargetAstBuilder<JavaDialect>,
    pub facade: JavaTypeDeclaration,
    pub record: JavaTypeDeclaration,
    pub record_id: GeneratedTypeId,
    pub facade_id: GeneratedTypeId,
}

#[derive(Clone, Copy)]
pub enum Facade {
    Harness,
    EntryPoint,
}

impl Facade {
    fn name(self) -> &'static str {
        match self {
            Self::Harness => "Fixture",
            Self::EntryPoint => "Generated",
        }
    }
    fn origin(self) -> GeneratedOrigin<JavaDialect> {
        GeneratedOrigin::Synthesized(match self {
            Self::Harness => SynthesisReason::TestHarness,
            Self::EntryPoint => SynthesisReason::PackageEntryPoint,
        })
    }
}

pub fn reference(owner: GeneratedTypeId, hash: u64, spelling: &str, ty: JavaType) -> JavaFieldRef {
    JavaFieldRef::RustSource {
        owner,
        field: id(hash),
        name: name(spelling),
        ty,
    }
}
pub fn field(receiver: JavaExpr, reference: JavaFieldRef, ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Field {
            receiver: Box::new(receiver),
            field: reference,
        },
    }
}
pub fn this(owner: GeneratedTypeId) -> JavaExpr {
    JavaExpr {
        ty: JavaType::Reference(JavaTypeName::Generated(owner)),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::This),
    }
}
pub fn method(spelling: &str, ty: JavaType, body: Vec<JavaStmt>) -> JavaMethod {
    JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public, JavaModifier::Static],
        type_parameters: vec![],
        return_type: ty,
        name: name(spelling),
        parameters: vec![],
        body: Some(JavaBlock::new(body)),
    }
}

impl Fixture {
    pub fn new() -> Self {
        Self::with_nominal_origin(None)
    }

    pub fn with_nominal_origin(replacement: Option<GeneratedOrigin<JavaDialect>>) -> Self {
        Self::build(replacement, origin(), Facade::Harness, int())
    }

    pub fn with_metadata(metadata: RustSourceOrigin, facade: Facade) -> Self {
        Self::build(None, metadata, facade, int())
    }

    pub fn with_double_metadata(metadata: RustSourceOrigin, facade: Facade) -> Self {
        Self::build(
            None,
            metadata,
            facade,
            JavaType::primitive(JavaPrimitive::Double),
        )
    }

    fn build(
        replacement: Option<GeneratedOrigin<JavaDialect>>,
        metadata: RustSourceOrigin,
        facade: Facade,
        value_type: JavaType,
    ) -> Self {
        let mut builder = TargetAstBuilder::new(JavaDialect);
        let facade_id = builder.generated_type(GeneratedType {
            name: facade.name().into(),
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            origin: facade.origin(),
            source: source(),
        });
        let record_id = builder.generated_type(GeneratedType {
            name: "Cell".into(),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Private,
            origin: replacement
                .unwrap_or_else(|| GeneratedOrigin::RustSource(Arc::new(metadata.clone()))),
            source: source(),
        });
        let mut parameters = Vec::new();
        let mut components = Vec::new();
        let mut assignments = Vec::new();
        for (hash, spelling, ty) in [(3, "value", value_type.clone()), (4, "flag", boolean())] {
            let mut field_origin = metadata.clone();
            field_origin.declaration = id(hash);
            parameters.push(JavaParameter {
                ty: ty.clone(),
                name: name(spelling),
                final_parameter: true,
            });
            components.push(JavaRecordComponent {
                origin: JavaRecordComponentOrigin::RustSource(JavaSourceFieldOrigin {
                    owner: id(2),
                    origin: Arc::new(field_origin),
                }),
                ty: ty.clone(),
                name: name(spelling),
            });
            assignments.push(JavaStmt::Assign {
                target: field(
                    this(record_id),
                    reference(record_id, hash, spelling, ty.clone()),
                    ty.clone(),
                ),
                value: JavaExpr::local(ty, name(spelling)),
            });
        }
        let record = JavaTypeDeclaration {
            declared: Some(record_id),
            kind: JavaDeclarationKind::Record,
            visibility: JavaVisibility::Private,
            modifiers: vec![],
            name: name("Cell"),
            type_parameters: vec![],
            record_components: components,
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Private],
                name: name("Cell"),
                parameters: parameters.clone(),
                body: JavaBlock::new(assignments),
            })],
        };
        let cell_type = JavaType::Reference(JavaTypeName::Generated(record_id));
        let cell = || JavaExpr::local(cell_type.clone(), name("cell"));
        let mut inspect = method(
            "inspect",
            value_type.clone(),
            vec![
                JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: cell_type.clone(),
                    name: name("cell"),
                    value: Some(JavaExpr {
                        ty: cell_type.clone(),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::New {
                            constructor: JavaConstructorRef::Generated {
                                owner: record_id,
                                parameters: vec![value_type.clone(), boolean()],
                            },
                            arguments: vec![
                                JavaExpr::local(value_type.clone(), name("value")),
                                JavaExpr::local(boolean(), name("flag")),
                            ],
                        },
                    }),
                },
                JavaStmt::Return(Some(JavaExpr {
                    ty: value_type.clone(),
                    precedence: JavaPrecedence::Conditional,
                    kind: JavaExprKind::Conditional {
                        condition: Box::new(field(
                            cell(),
                            reference(record_id, 4, "flag", boolean()),
                            boolean(),
                        )),
                        when_true: Box::new(field(
                            cell(),
                            reference(record_id, 3, "value", value_type.clone()),
                            value_type.clone(),
                        )),
                        when_false: Box::new(JavaExpr::literal(
                            value_type.clone(),
                            if value_type == int() {
                                JavaLiteral::I32(-1)
                            } else {
                                JavaLiteral::F64(
                                    portable_binary64::FiniteBinary64::from_bits(
                                        0xbff0_0000_0000_0000,
                                    )
                                    .unwrap(),
                                )
                            },
                        )),
                    },
                })),
            ],
        );
        inspect.parameters = parameters;
        let facade = JavaTypeDeclaration {
            declared: Some(facade_id),
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            modifiers: vec![],
            name: name(facade.name()),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![JavaMember::Method(inspect)],
        };
        Self {
            builder,
            facade,
            record,
            record_id,
            facade_id,
        }
    }

    pub fn constructor(&mut self) -> &mut JavaConstructor {
        let JavaMember::Constructor(value) = &mut self.record.members[0] else {
            panic!("constructor");
        };
        value
    }

    pub fn finish(mut self) -> TargetAstPackage<JavaDialect> {
        self.facade
            .members
            .push(JavaMember::NestedType(self.record));
        let filename = self.facade.name.as_str().to_owned();
        let mut declared = vec![
            GeneratedSymbolId::Type(self.facade_id),
            GeneratedSymbolId::Type(self.record_id),
        ];
        declared.extend(
            self.facade
                .members
                .iter()
                .filter_map(|member| match member {
                    JavaMember::Method(method) => match method.declared {
                        JavaMethodDeclaration::Callable(id) => {
                            Some(GeneratedSymbolId::Callable(id))
                        }
                        _ => None,
                    },
                    _ => None,
                }),
        );
        add_file(&mut self.builder, &filename, declared, self.facade);
        self.builder.build()
    }
}

pub fn add_file(
    builder: &mut TargetAstBuilder<JavaDialect>,
    filename: &str,
    declared: Vec<GeneratedSymbolId>,
    declaration: JavaTypeDeclaration,
) {
    let module = JavaPackage::RustCrate(7);
    let file = builder.file(TargetFile::new(
        RelativeOutputPath::new(format!(
            "{}{filename}.java",
            module.source_directory(JavaFilePlacement::Main)
        ))
        .unwrap(),
        SourceRole::PublicApi,
        module,
        JavaFilePlacement::Main,
        vec![JavaFileItem::Type {
            declared,
            conformances: JavaConformanceInventory::structural().into(),
            package_metadata: None,
            dependencies: Default::default(),
            declaration: Box::new(declaration),
        }],
        JavaSourceFileKind::CompilationUnit,
        source(),
    ));
    builder.group(TargetFileGroup::new(
        FileGroupRole::PublicApi,
        vec![TargetFileMember::Source(file)],
        source(),
    ));
}

pub fn certify(package: TargetAstPackage<JavaDialect>) -> RenderedPackage {
    let verified = verify_unresolved_package(&JavaDialect, package).unwrap();
    let linked = TargetLinker::new(JavaDialect).link_ast(&verified).unwrap();
    let certificate = certify_resolved_package(&JavaDialect, linked).unwrap();
    let output = render_certified_package(&crate::render::JavaRenderer, &certificate).unwrap();
    for _ in 0..2 {
        assert_eq!(
            output,
            render_certified_package(&crate::render::JavaRenderer, &certificate).unwrap()
        );
    }
    output
}
