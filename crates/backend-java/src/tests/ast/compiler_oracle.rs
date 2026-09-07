use super::{
    GeneratedSymbolId, JavaAnnotation, JavaArrayOwnership, JavaBlock, JavaCallableRef,
    JavaDeclarationKind, JavaDialect, JavaEnumConstant, JavaExpr, JavaExprKind, JavaField,
    JavaFileItem, JavaFilePlacement, JavaHeritage, JavaIdentifier, JavaKnownMethod, JavaKnownType,
    JavaLiteral, JavaMember, JavaMemberOrigin, JavaMethod, JavaMethodDeclaration, JavaModifier,
    JavaPackage, JavaPattern, JavaPrecedence, JavaPrimitive, JavaRecordComponent,
    JavaRecordComponentOrigin, JavaRuntimeMember, JavaSourceFileKind, JavaStmt, JavaSwitchArm,
    JavaType, JavaTypeDeclaration, JavaTypeName, JavaVisibility, JavaWildcardBound, TargetTypeRef,
    parameter, structural_method, verifier_source,
};

#[test]
fn verified_java_mutation_corpus_compiles_under_hermetic_java_21() {
    let Some(test_tmpdir) = std::env::var_os("TEST_TMPDIR") else {
        eprintln!("Java mutation/compiler oracle runs under the authoritative Bazel target");
        return;
    };
    let mut state = 0x6a09_e667_f3bc_c909_u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let output_root = std::path::PathBuf::from(test_tmpdir).join("java-ast-mutation-oracle");
    let classes = output_root.join("classes");
    std::fs::create_dir_all(&classes).expect("create javac output directory");
    let mut sources = Vec::new();
    let mut rejected = 0usize;

    for index in 0..128 {
        let name = format!("OracleCase{index}");
        let field_modifiers = match next() % 8 {
            0 => vec![],
            1 => vec![JavaModifier::Private],
            2 => vec![JavaModifier::Public],
            3 => vec![JavaModifier::Static],
            4 => vec![JavaModifier::Final],
            5 => vec![JavaModifier::Private, JavaModifier::Final],
            6 => vec![JavaModifier::Private, JavaModifier::Static],
            _ => vec![
                JavaModifier::Private,
                JavaModifier::Static,
                JavaModifier::Final,
            ],
        };
        let field_initializer = (next() % 3 != 0).then(|| {
            JavaExpr::literal(
                JavaType::primitive(JavaPrimitive::Int),
                JavaLiteral::I32(index),
            )
        });
        let method_modifiers = match next() % 8 {
            0 => vec![],
            1 => vec![JavaModifier::Public],
            2 => vec![JavaModifier::Private],
            3 => vec![JavaModifier::Static],
            4 => vec![JavaModifier::Public, JavaModifier::Static],
            5 => vec![JavaModifier::Public, JavaModifier::Final],
            6 => vec![JavaModifier::Public, JavaModifier::Abstract],
            _ => vec![JavaModifier::Private, JavaModifier::Abstract],
        };
        let annotations = match next() % 8 {
            0 => vec![JavaAnnotation::Override],
            1 => vec![JavaAnnotation::SafeVarargs],
            2 => vec![JavaAnnotation::Override, JavaAnnotation::Override],
            _ => vec![],
        };
        let parameter_type = match next() % 10 {
            0 => JavaType::primitive(JavaPrimitive::Int),
            1 => JavaType::known(JavaKnownType::String),
            2 => JavaType::known(JavaKnownType::Integer),
            3 => JavaType::generic(
                JavaKnownType::List,
                vec![JavaType::known(JavaKnownType::String)],
            ),
            4 => JavaType::generic(
                JavaKnownType::List,
                vec![JavaType::known(JavaKnownType::Integer)],
            ),
            5 => JavaType::generic(
                JavaKnownType::List,
                vec![JavaType::Wildcard { bound: None }],
            ),
            6 => JavaType::generic(
                JavaKnownType::List,
                vec![JavaType::Wildcard {
                    bound: Some((
                        JavaWildcardBound::Extends,
                        Box::new(JavaType::known(JavaKnownType::String)),
                    )),
                }],
            ),
            7 => JavaType::Array {
                component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
                ownership: JavaArrayOwnership::DefensiveCopyBoundary,
            },
            8 => JavaType::generic(
                JavaKnownType::List,
                vec![JavaType::generic(
                    JavaKnownType::List,
                    vec![JavaType::known(JavaKnownType::String)],
                )],
            ),
            _ => JavaType::generic(
                JavaKnownType::List,
                vec![JavaType::Array {
                    component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
                    ownership: JavaArrayOwnership::InternalMutable,
                }],
            ),
        };
        let abstract_method = method_modifiers.contains(&JavaModifier::Abstract);
        let declaration = JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable(&name),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![
                JavaMember::Field(JavaField {
                    declared: None,
                    modifiers: field_modifiers,
                    ty: JavaType::primitive(JavaPrimitive::Int),
                    name: JavaIdentifier::from_portable("value"),
                    initializer: field_initializer,
                }),
                JavaMember::Method(JavaMethod {
                    declared: JavaMethodDeclaration::Structural,
                    annotations,
                    modifiers: method_modifiers,
                    type_parameters: vec![],
                    return_type: JavaType::primitive(JavaPrimitive::Void),
                    name: JavaIdentifier::from_portable("accept"),
                    parameters: vec![parameter(parameter_type, "input")],
                    body: (!abstract_method).then(|| JavaBlock::new(vec![])),
                }),
            ],
        };
        let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
        let file = builder.file(portable_codegen::TargetFile::new(
            portable_codegen::RelativeOutputPath::new(format!(
                "src/main/java/org/polyrust/generated/{name}.java"
            ))
            .unwrap(),
            portable_codegen::SourceRole::PublicApi,
            JavaPackage::Generated,
            JavaFilePlacement::Main,
            vec![JavaFileItem::Type {
                declared: vec![],
                declaration,
            }],
            JavaSourceFileKind::CompilationUnit,
            verifier_source("mutation-oracle-file"),
        ));
        builder.group(portable_codegen::TargetFileGroup::new(
            portable_codegen::FileGroupRole::PublicApi,
            vec![portable_codegen::TargetFileMember::Source(file)],
            verifier_source("mutation-oracle-group"),
        ));
        let package = builder.build();
        let verified = match portable_codegen::verify_unresolved_package(&JavaDialect, package) {
            Ok(verified) => verified,
            Err(_) => {
                rejected += 1;
                continue;
            }
        };
        let linked = portable_codegen::TargetLinker::new(JavaDialect)
            .link_ast(&verified)
            .expect("every verified mutation must link");
        let certified = portable_codegen::certify_resolved_package(&JavaDialect, linked)
            .expect("every linked mutation must become render-ready");
        let rendered =
            portable_codegen::render_certified_package(&crate::render::JavaRenderer, &certified)
                .expect("every render-ready mutation must render");
        for file in rendered.files() {
            let portable_codegen::OutputContents::Text(contents) = file.contents() else {
                panic!("Java mutation rendered non-text output")
            };
            let path = output_root.join(file.path());
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("create Java source directory");
            }
            std::fs::write(&path, contents).expect("write verifier-accepted Java source");
            sources.push(path);
        }
    }
    assert!(
        sources.len() >= 16,
        "mutation corpus accepted too few cases"
    );
    assert!(rejected >= 16, "mutation corpus rejected too few cases");

    let string = JavaType::known(JavaKnownType::String);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let type_variable = JavaIdentifier::from_portable("T");
    let bounded_strings = JavaType::generic(
        JavaKnownType::List,
        vec![JavaType::Wildcard {
            bound: Some((JavaWildcardBound::Extends, Box::new(string.clone()))),
        }],
    );
    let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
    let oracle_enum = builder.generated_type(portable_codegen::GeneratedType {
        name: "OracleChoice".to_owned(),
        kind: JavaDeclarationKind::Enum,
        visibility: JavaVisibility::Public,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::TestHarness,
        ),
        source: verifier_source("oracle-enum"),
    });
    let oracle_enum_type = JavaType::Reference(JavaTypeName::Generated(oracle_enum));
    let oracle_enum_target = TargetTypeRef::Generated(oracle_enum);
    let oracle_first = builder.value(portable_codegen::GeneratedValue {
        name: "FIRST".to_owned(),
        ty: oracle_enum_target.clone(),
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::TestHarness,
        ),
        source: verifier_source("oracle-first"),
    });
    let oracle_second = builder.value(portable_codegen::GeneratedValue {
        name: "SECOND".to_owned(),
        ty: oracle_enum_target,
        origin: portable_codegen::GeneratedOrigin::Synthesized(
            portable_codegen::SynthesisReason::TestHarness,
        ),
        source: verifier_source("oracle-second"),
    });
    let structured_enum = JavaTypeDeclaration {
        declared: Some(oracle_enum),
        kind: JavaDeclarationKind::Enum,
        visibility: JavaVisibility::Public,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("OracleChoice"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![
            JavaMember::EnumConstant(JavaEnumConstant {
                declared: oracle_first,
                name: JavaIdentifier::from_portable("FIRST"),
            }),
            JavaMember::EnumConstant(JavaEnumConstant {
                declared: oracle_second,
                name: JavaIdentifier::from_portable("SECOND"),
            }),
        ],
    };
    let structured_record = JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Record,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("OracleStructuredRecord"),
        type_parameters: vec![],
        record_components: vec![JavaRecordComponent {
            origin: JavaRecordComponentOrigin::Runtime(JavaRuntimeMember::ErrorCode),
            ty: string.clone(),
            name: JavaIdentifier::from_portable("value"),
        }],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![],
    };
    let structured_interface = JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Package,
        modifiers: vec![],
        name: JavaIdentifier::from_portable("OracleStructuredInterface"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![
            JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Structural,
                annotations: vec![],
                modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
                type_parameters: vec![type_variable.clone()],
                return_type: JavaType::TypeVariable(type_variable.clone()),
                name: JavaIdentifier::from_portable("identity"),
                parameters: vec![parameter(JavaType::TypeVariable(type_variable), "value")],
                body: None,
            }),
            JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Structural,
                annotations: vec![],
                modifiers: vec![JavaModifier::Public, JavaModifier::Abstract],
                type_parameters: vec![],
                return_type: JavaType::primitive(JavaPrimitive::Void),
                name: JavaIdentifier::from_portable("accept"),
                parameters: vec![parameter(bounded_strings, "values")],
                body: None,
            }),
        ],
    };
    let length_call = JavaExpr {
        ty: int.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: string.clone(),
                name: JavaIdentifier::from_portable(JavaKnownMethod::StringLength.name().text()),
                signature: JavaKnownMethod::StringLength.signature(),
                origin: JavaMemberOrigin::Known(JavaKnownMethod::StringLength),
            },
            receiver: Some(Box::new(JavaExpr::literal(
                string.clone(),
                JavaLiteral::String("oracle".to_owned()),
            ))),
            arguments: vec![],
        },
    };
    let structured_class =
        JavaTypeDeclaration {
            declared: None,
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Package,
            modifiers: vec![],
            name: JavaIdentifier::from_portable("OracleStructured"),
            type_parameters: vec![],
            record_components: vec![],
            heritage: JavaHeritage::None,
            permits: vec![],
            members: vec![
                JavaMember::NestedType(structured_enum),
                JavaMember::Field(JavaField {
                    declared: None,
                    modifiers: vec![JavaModifier::Private],
                    ty: JavaType::Array {
                        component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
                        ownership: JavaArrayOwnership::DefensiveCopyBoundary,
                    },
                    name: JavaIdentifier::from_portable("bytes"),
                    initializer: None,
                }),
                JavaMember::NestedType(JavaTypeDeclaration {
                    declared: None,
                    kind: JavaDeclarationKind::FinalClass,
                    visibility: JavaVisibility::Private,
                    modifiers: vec![JavaModifier::Static],
                    name: JavaIdentifier::from_portable("Inner"),
                    type_parameters: vec![],
                    record_components: vec![],
                    heritage: JavaHeritage::None,
                    permits: vec![],
                    members: vec![],
                }),
                structural_method(
                    "length",
                    int.clone(),
                    vec![],
                    JavaBlock::new(vec![JavaStmt::Return(Some(length_call))]),
                ),
                structural_method(
                    "widen",
                    JavaType::primitive(JavaPrimitive::Long),
                    vec![],
                    JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                        ty: JavaType::primitive(JavaPrimitive::Long),
                        precedence: JavaPrecedence::Unary,
                        kind: JavaExprKind::Cast {
                            target: JavaType::primitive(JavaPrimitive::Long),
                            value: Box::new(JavaExpr::literal(int.clone(), JavaLiteral::I32(1))),
                        },
                    }))]),
                ),
                structural_method(
                    "branch",
                    int.clone(),
                    vec![parameter(boolean.clone(), "flag")],
                    JavaBlock::new(vec![JavaStmt::If {
                        condition: JavaExpr::local(boolean, JavaIdentifier::from_portable("flag")),
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(
                            JavaExpr::literal(int.clone(), JavaLiteral::I32(1)),
                        ))]),
                        else_block: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
                            JavaExpr::literal(int.clone(), JavaLiteral::I32(2)),
                        ))])),
                    }]),
                ),
                structural_method(
                    "enumRank",
                    int.clone(),
                    vec![parameter(oracle_enum_type.clone(), "value")],
                    JavaBlock::new(vec![JavaStmt::Switch {
                        value: JavaExpr::local(
                            oracle_enum_type,
                            JavaIdentifier::from_portable("value"),
                        ),
                        arms: vec![
                            JavaSwitchArm {
                                pattern: JavaPattern::EnumVariant {
                                    enumeration: oracle_enum,
                                    variant: oracle_first,
                                },
                                body: JavaBlock::new(vec![JavaStmt::Return(Some(
                                    JavaExpr::literal(int.clone(), JavaLiteral::I32(1)),
                                ))]),
                            },
                            JavaSwitchArm {
                                pattern: JavaPattern::EnumVariant {
                                    enumeration: oracle_enum,
                                    variant: oracle_second,
                                },
                                body: JavaBlock::new(vec![JavaStmt::Return(Some(
                                    JavaExpr::literal(int.clone(), JavaLiteral::I32(2)),
                                ))]),
                            },
                            JavaSwitchArm {
                                pattern: JavaPattern::Default,
                                body: JavaBlock::new(vec![JavaStmt::ThrowAssertion(
                                    JavaExpr::literal(
                                        string.clone(),
                                        JavaLiteral::String("unreachable".to_owned()),
                                    ),
                                )]),
                            },
                        ],
                    }]),
                ),
            ],
        };
    let file = builder.file(portable_codegen::TargetFile::new(
        portable_codegen::RelativeOutputPath::new(
            "src/main/java/org/polyrust/generated/OracleStructured.java",
        )
        .unwrap(),
        portable_codegen::SourceRole::PublicApi,
        JavaPackage::Generated,
        JavaFilePlacement::Main,
        vec![
            JavaFileItem::Type {
                declared: vec![],
                declaration: structured_record,
            },
            JavaFileItem::Type {
                declared: vec![],
                declaration: structured_interface,
            },
            JavaFileItem::Type {
                declared: vec![
                    GeneratedSymbolId::Type(oracle_enum),
                    GeneratedSymbolId::Value(oracle_first),
                    GeneratedSymbolId::Value(oracle_second),
                ],
                declaration: structured_class,
            },
        ],
        JavaSourceFileKind::CompilationUnit,
        verifier_source("structured-mutation-oracle-file"),
    ));
    builder.group(portable_codegen::TargetFileGroup::new(
        portable_codegen::FileGroupRole::PublicApi,
        vec![portable_codegen::TargetFileMember::Source(file)],
        verifier_source("structured-mutation-oracle-group"),
    ));
    let package = builder.build();
    let verified = portable_codegen::verify_unresolved_package(&JavaDialect, package)
        .expect("structured Java mutation package must verify");
    let linked = portable_codegen::TargetLinker::new(JavaDialect)
        .link_ast(&verified)
        .expect("structured Java mutation package must link");
    let certified = portable_codegen::certify_resolved_package(&JavaDialect, linked)
        .expect("structured Java mutation package must become render-ready");
    let rendered =
        portable_codegen::render_certified_package(&crate::render::JavaRenderer, &certified)
            .expect("structured render-ready Java mutation package must render");
    for file in rendered.files() {
        let portable_codegen::OutputContents::Text(contents) = file.contents() else {
            panic!("structured Java mutation rendered non-text output")
        };
        let path = output_root.join(file.path());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create structured Java source directory");
        }
        std::fs::write(&path, contents).expect("write structured Java mutation source");
        sources.push(path);
    }

    let runfiles = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .map(std::path::PathBuf::from)
        .expect("Bazel supplies a runfiles root");
    let javac = std::fs::read_dir(&runfiles)
        .expect("read runfiles root")
        .filter_map(Result::ok)
        .find_map(|entry| {
            let name = entry.file_name();
            name.to_string_lossy()
                .contains("remotejdk21")
                .then(|| entry.path().join("bin/javac"))
                .filter(|candidate| candidate.is_file())
        })
        .expect("hermetic Java 21 javac is present in runfiles");
    let output = std::process::Command::new(javac)
        .args(["--release", "21", "-Werror", "-Xlint:all", "-d"])
        .arg(&classes)
        .args(&sources)
        .output()
        .expect("run hermetic javac over verifier-accepted mutations");
    assert!(
        output.status.success(),
        "verified Java AST mutation failed javac:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
