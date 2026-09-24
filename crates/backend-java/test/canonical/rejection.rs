//! Profile errors reject before a renderer or dependency API receives a witness.
use super::{fixture::*, model::*};
use portable_backend_java::ast::*;
use portable_codegen::*;

fn rejected(owner: Owner, message: &str) {
    let error = certify(owner.finish()).unwrap_err();
    assert!(error.contains(message), "{error}");
}

#[test]
fn canonical_namespace_and_metadata_must_agree() {
    let owner = Owner::new(false);
    assert!(
        certify(owner.with_metadata(Some(None)))
            .unwrap_err()
            .contains("explicit canonical")
    );
    for namespace in [
        JavaPackage::Generated,
        JavaPackage::RustCrate(7),
        Owner::new(true).namespace,
    ] {
        let mut owner = Owner::new(false);
        owner.namespace = namespace;
        owner.path = format!(
            "{}Generated.java",
            namespace.source_directory(JavaFilePlacement::Main)
        );
        rejected(owner, "namespace");
    }
    let mut owner = Owner::new(false);
    owner.path = owner.path.replace("Generated.java", "Other.java");
    rejected(owner, "source path");
    let owner = Owner::new(false);
    let metadata = JavaCanonicalTypePackage::new(
        owner.metadata.profile(),
        facts(true),
        owner.fixture.types,
        owner.fixture.kinds,
    );
    assert!(
        certify(owner.with_metadata(Some(Some(metadata.into()))))
            .unwrap_err()
            .contains("namespace")
    );
}

#[test]
fn canonical_namespace_cannot_impersonate_a_source_crate_with_documentation() {
    let root = RustDeclarationId {
        crate_id: 7,
        definition_path_hash: 0,
    };
    let module = std::sync::Arc::new(RustModuleDocumentation {
        declaration: root,
        parent: None,
        location: RustSourceLocation {
            file: "lib.rs".into(),
            line: 1,
            column: 1,
        },
        documentation: vec!["Caller supplied facade documentation".into()],
    });
    let exports = RustCrateExports {
        root,
        modules: std::collections::BTreeMap::from([(root, Default::default())]),
        module_ancestries: std::collections::BTreeMap::from([(root, vec![module].into())]),
    };
    let metadata = JavaSourcePackage::new(std::sync::Arc::new(exports));
    let error = certify(Owner::new(false).with_metadata(Some(Some(metadata.into())))).unwrap_err();
    assert!(error.contains("explicit canonical owner"), "{error}");
}

#[test]
fn role_selections_reject_swaps_duplicates_and_wrong_types() {
    for mutation in 0..3 {
        let mut owner = Owner::new(false);
        let mut kinds = owner.fixture.kinds;
        let mut types = owner.fixture.types;
        match mutation {
            0 => kinds.zero = kinds.empty,
            1 => std::mem::swap(&mut kinds.positive_overflow, &mut kinds.negative_overflow),
            2 => types.error = types.success,
            _ => unreachable!(),
        }
        owner.metadata = JavaCanonicalTypePackage::new(
            owner.metadata.profile(),
            owner.metadata.facts(),
            types,
            kinds,
        );
        assert!(certify(owner.finish()).is_err(), "mutation {mutation}");
    }
}

#[test]
fn exact_family_and_facade_are_not_extensible_by_callers() {
    for mutation in 0..13 {
        let mut owner = Owner::new(false);
        let fixture = &mut owner.fixture;
        match mutation {
            0 => fixture.facade.members.clear(),
            1 => {
                let JavaMember::Constructor(value) = &mut fixture.facade.members[0] else {
                    panic!("constructor")
                };
                value.modifiers = vec![JavaModifier::Public];
            }
            2 => {
                let JavaMember::Constructor(value) = &mut fixture.facade.members[0] else {
                    panic!("constructor")
                };
                value.body.statements.push(JavaStmt::Return(None));
            }
            3 => fixture.facade.modifiers.push(JavaModifier::Static),
            4 => fixture.facade.members.push(JavaMember::Field(JavaField {
                declared: None,
                modifiers: vec![
                    JavaModifier::Private,
                    JavaModifier::Static,
                    JavaModifier::Final,
                ],
                ty: int(),
                name: name("extra"),
                initializer: Some(JavaExpr::literal(int(), JavaLiteral::I32(1))),
            })),
            5 => fixture.interface.permits.reverse(),
            6 => fixture.error.members.reverse(),
            7 => fixture.error.heritage = JavaHeritage::None,
            8 => {
                fixture.error.members.pop();
            }
            9 => fixture.success.record_components[0].name = name("other"),
            10 => {
                let JavaMember::Constructor(value) = &mut fixture.success.members[0] else {
                    panic!("constructor")
                };
                value.parameters[0].final_parameter = false;
            }
            11 => {
                register(
                    &mut fixture.builder,
                    "Unused",
                    JavaDeclarationKind::FinalClass,
                );
            }
            12 => {
                fixture.builder.value(GeneratedValue {
                    name: "UNUSED".into(),
                    ty: TargetTypeRef::Generated(fixture.types.error),
                    visibility: JavaVisibility::Public,
                    origin: GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter),
                    source: source(),
                });
            }
            _ => unreachable!(),
        }
        assert!(certify(owner.finish()).is_err(), "mutation {mutation}");
    }
}

#[test]
fn extra_files_and_groups_cannot_hide_in_a_type_only_owner() {
    let mut owner = Owner::new(false);
    owner.fixture.builder.group(TargetFileGroup::new(
        FileGroupRole::PublicApi,
        vec![],
        source(),
    ));
    rejected(owner, "one source file and public group");
    let mut owner = Owner::new(false);
    owner.fixture.builder.file(TargetFile::new(
        RelativeOutputPath::new("src/main/java/org/polyrust/generated/Other.java").unwrap(),
        SourceRole::PublicApi,
        JavaPackage::Generated,
        JavaFilePlacement::Main,
        vec![],
        JavaSourceFileKind::CompilationUnit,
        source(),
    ));
    rejected(owner, "one source file and public group");
}
