//! Target package syntax/layout evidence, not compiler-source provenance.
use crate::ast::*;
use crate::dialect::JavaDialect;
use portable_codegen::*;
use portable_diagnostics::SourceRef;

fn source() -> SourceRef {
    SourceRef::logical(["java-crate-package-test"])
}

fn fixture(package: JavaPackage, name: &str, path: String) -> TargetFile<JavaDialect> {
    TargetFile::new(
        RelativeOutputPath::new(path).unwrap(),
        SourceRole::PublicApi,
        package,
        JavaFilePlacement::Main,
        vec![JavaFileItem::Type {
            declared: vec![],
            conformances: JavaConformanceInventory::structural().into(),
            source_package: None,
            dependencies: Default::default(),
            declaration: Box::new(JavaTypeDeclaration {
                declared: None,
                kind: JavaDeclarationKind::FinalClass,
                visibility: JavaVisibility::Public,
                modifiers: vec![],
                name: JavaIdentifier::new(name).unwrap(),
                type_parameters: vec![],
                record_components: vec![],
                heritage: JavaHeritage::None,
                permits: vec![],
                members: vec![],
            }),
        }],
        JavaSourceFileKind::CompilationUnit,
        source(),
    )
}

fn package(files: Vec<TargetFile<JavaDialect>>) -> TargetAstPackage<JavaDialect> {
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let members = files
        .into_iter()
        .map(|file| TargetFileMember::Source(builder.file(file)))
        .collect();
    builder.group(TargetFileGroup::new(
        FileGroupRole::PublicApi,
        members,
        source(),
    ));
    builder.build()
}

fn valid(module: JavaPackage) -> TargetAstPackage<JavaDialect> {
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let owner = builder.generated_type(GeneratedType {
        name: "Fixture".into(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
        source: source(),
    });
    let ty = JavaType::Reference(JavaTypeName::Generated(owner));
    let declaration = JavaTypeDeclaration {
        declared: Some(owner),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        modifiers: vec![],
        name: JavaIdentifier::new("Fixture").unwrap(),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![
            JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Public],
                name: JavaIdentifier::new("Fixture").unwrap(),
                parameters: vec![],
                body: JavaBlock::new(vec![]),
            }),
            JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Structural,
                annotations: vec![],
                modifiers: vec![JavaModifier::Public, JavaModifier::Static],
                type_parameters: vec![],
                return_type: ty.clone(),
                name: JavaIdentifier::new("make").unwrap(),
                parameters: vec![],
                body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                    ty,
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::New {
                        constructor: JavaConstructorRef::Generated {
                            owner,
                            parameters: vec![],
                        },
                        arguments: vec![],
                    },
                }))])),
            }),
        ],
    };
    let path = format!(
        "{}Fixture.java",
        module.source_directory(JavaFilePlacement::Main)
    );
    let file = builder.file(TargetFile::new(
        RelativeOutputPath::new(path).unwrap(),
        SourceRole::PublicApi,
        module,
        JavaFilePlacement::Main,
        vec![JavaFileItem::Type {
            declared: vec![GeneratedSymbolId::Type(owner)],
            conformances: JavaConformanceInventory::structural().into(),
            source_package: None,
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
    builder.build()
}

fn certify(ast: TargetAstPackage<JavaDialect>) -> RenderReadyPackage<JavaDialect> {
    let verified = verify_unresolved_package(&JavaDialect, ast).expect("verify package");
    let linked = TargetLinker::new(JavaDialect)
        .link_ast(&verified)
        .expect("link package");
    certify_resolved_package(&JavaDialect, linked).expect("certify package")
}

#[test]
fn typed_package_spellings_and_roots_are_exact() {
    assert_eq!(JavaPackage::Generated.name(), "org.polyrust.generated");
    for (id, suffix) in [
        (0, "0000000000000000"),
        (1, "0000000000000001"),
        (u64::MAX, "ffffffffffffffff"),
    ] {
        let module = JavaPackage::RustCrate(id);
        assert_eq!(module.name(), format!("org.polyrust.generated.r{suffix}"));
        for placement in [JavaFilePlacement::Main, JavaFilePlacement::Runtime] {
            assert_eq!(
                module.source_directory(placement),
                format!("src/main/java/org/polyrust/generated/r{suffix}/")
            );
        }
        for placement in [
            JavaFilePlacement::NativeTest,
            JavaFilePlacement::Conformance,
            JavaFilePlacement::NegativeTest,
        ] {
            assert_eq!(
                module.source_directory(placement),
                format!("src/test/java/org/polyrust/generated/r{suffix}/")
            );
        }
    }
}

#[test]
fn wrong_paths_and_mixed_namespaces_reject_before_certification() {
    let module = JavaPackage::RustCrate(1);
    for path in [
        "src/main/java/org/polyrust/generated/Fixture.java",
        "src/main/java/org/polyrust/generated/r0000000000000002/Fixture.java",
        "src/main/java/org/polyrust/generated/r0000000000000001/nested/Fixture.java",
        "src/test/java/org/polyrust/generated/r0000000000000001/Fixture.java",
    ] {
        assert!(
            verify_unresolved_package(
                &JavaDialect,
                package(vec![fixture(module, "Fixture", path.into())])
            )
            .is_err(),
            "{path}"
        );
    }
    for other in [JavaPackage::Generated, JavaPackage::RustCrate(2)] {
        let files = [module, other]
            .into_iter()
            .enumerate()
            .map(|(index, module)| {
                let name = format!("Fixture{index}");
                fixture(
                    module,
                    &name,
                    format!(
                        "{}{name}.java",
                        module.source_directory(JavaFilePlacement::Main)
                    ),
                )
            })
            .collect();
        let errors = verify_unresolved_package(&JavaDialect, package(files)).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("exactly one namespace")),
            "{errors:?}"
        );
    }
}

#[test]
fn separately_certified_crate_namespaces_compile_together_deterministically() {
    let Some(temp) = std::env::var_os("TEST_TMPDIR") else {
        return; // Native oracle is authoritative under Bazel.
    };
    let output = std::path::PathBuf::from(temp).join("java-crate-packages");
    let classes = output.join("classes");
    std::fs::create_dir_all(&classes).unwrap();
    let mut sources = Vec::new();
    for module in [
        JavaPackage::Generated,
        JavaPackage::RustCrate(0),
        JavaPackage::RustCrate(1),
        JavaPackage::RustCrate(u64::MAX),
    ] {
        let certified = certify(valid(module));
        let rendered = render_certified_package(&crate::render::JavaRenderer, &certified).unwrap();
        for _ in 0..2 {
            let repeated =
                render_certified_package(&crate::render::JavaRenderer, &certified).unwrap();
            assert_eq!(rendered, repeated);
        }
        for file in rendered.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("expected Java text")
            };
            assert!(text.contains(&format!("package {};", module.name())));
            assert!(text.contains(&format!("new {}.Fixture()", module.name())));
            match module {
                JavaPackage::Generated => {
                    assert!(text.starts_with("// Generated by PolyRust from verified CoreIR.\n"));
                }
                JavaPackage::RustCrate(_) => {
                    assert!(text.starts_with("// Generated by PolyRust.\n"));
                    assert!(!text.contains("verified CoreIR"));
                }
            }
            let path = output.join(file.path());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, text).unwrap();
            sources.push(path);
        }
    }
    let runfiles = std::env::var_os("RUNFILES_DIR")
        .or_else(|| std::env::var_os("TEST_SRCDIR"))
        .unwrap();
    let javac = std::fs::read_dir(runfiles)
        .unwrap()
        .filter_map(Result::ok)
        .find_map(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("remotejdk21")
                .then(|| entry.path().join("bin/javac"))
                .filter(|path| path.is_file())
        })
        .expect("pinned Java 21 javac");
    let result = std::process::Command::new(javac)
        .args(["--release", "21", "-Xlint:all", "-Werror", "-d"])
        .arg(&classes)
        .args(&sources)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    for module in [
        JavaPackage::Generated,
        JavaPackage::RustCrate(0),
        JavaPackage::RustCrate(1),
        JavaPackage::RustCrate(u64::MAX),
    ] {
        assert!(
            classes
                .join(module.name().replace('.', "/"))
                .join("Fixture.class")
                .is_file()
        );
    }
}

#[test]
fn runtime_cannot_be_relocated_into_a_crate_namespace() {
    for (role, placement, group) in [
        (
            SourceRole::Runtime,
            JavaFilePlacement::Runtime,
            FileGroupRole::Runtime,
        ),
        (
            SourceRole::PublicApi,
            JavaFilePlacement::Main,
            FileGroupRole::PublicApi,
        ),
        (
            SourceRole::Implementation,
            JavaFilePlacement::Main,
            FileGroupRole::Implementation,
        ),
    ] {
        let mut builder = TargetAstBuilder::new(JavaDialect);
        let module = JavaPackage::RustCrate(1);
        let file = builder.file(TargetFile::new(
            RelativeOutputPath::new(format!(
                "{}Runtime.java",
                module.source_directory(placement)
            ))
            .unwrap(),
            role,
            module,
            placement,
            vec![crate::runtime::shell_item()],
            JavaSourceFileKind::CompilationUnit,
            source(),
        ));
        builder.group(TargetFileGroup::new(
            group,
            vec![TargetFileMember::Source(file)],
            source(),
        ));
        let errors = verify_unresolved_package(&JavaDialect, builder.build()).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Runtime identity requires")),
            "{role:?}/{placement:?}: {errors:?}"
        );
    }
}

#[test]
fn empty_certificate_renders_no_namespace_or_files() {
    let certified = certify(TargetAstBuilder::new(JavaDialect).build());
    let rendered = render_certified_package(&crate::render::JavaRenderer, &certified).unwrap();
    assert!(rendered.files().is_empty());
}
