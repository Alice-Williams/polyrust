use super::{source_constant_fixture as c, source_dependency_fixture as f};
use crate::{ast::*, dialect::*};
use portable_codegen::*;

#[test]
fn constant_only_and_mixed_apis_preserve_exact_witnesses_and_source() {
    for mixed in [false, true] {
        let api = c::api(mixed);
        assert_eq!(api.constants().len(), 8);
        assert_eq!(api.functions().len(), if mixed { 8 } else { 0 });
        assert_eq!(api.dependencies().len(), 0);
        let descriptions = api.source_descriptions().unwrap();
        assert_eq!(descriptions.len(), if mixed { 16 } else { 8 });
        for (i, expected) in c::values().iter().enumerate() {
            let id = f::id(0x35c, 10 + i as u64);
            let constant = api.constant(id).unwrap();
            assert_eq!(constant.value(), expected);
            assert_eq!(
                constant.source().documentation,
                [format!(" Constant {i} documentation.")]
            );
            assert_eq!(constant.ty(), &c::ty(expected));
            assert_eq!(constant.package_identity(), api.package_identity());
            assert_eq!(constant.path().package(), JavaPackage::RustCrate(0x35c));
            assert_eq!(constant.path().owners(), &[f::name("Generated")]);
            assert_eq!(constant.path().member(), &f::name(&format!("constant{i}")));
            assert!(api.function(id).is_none());
            assert_eq!(
                constant.source().crate_exports.modules[&api.root()]
                    .values()
                    .filter(|target| **target == RustExportTarget::Declaration(id))
                    .count(),
                2
            );
            let description = descriptions
                .iter()
                .find(|d| d.source().declaration == id)
                .unwrap();
            assert_eq!(
                description.kind(),
                JavaSourceDescriptionKind::Constant {
                    ty: constant.ty(),
                    value: constant.value()
                }
            );
            assert_eq!(
                description.target(),
                JavaSourceTarget::Declaration(constant.path())
            );
        }
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        assert_eq!(output.files().len(), 1);
        let OutputContents::Text(text) = output.files()[0].contents() else {
            panic!("source")
        };
        assert!(text.len() as u64 <= api.source_byte_bound().unwrap());
        assert_eq!(text.matches("Source owner documentation").count(), 1);
        assert!(!text.contains("Runtime") && !text.contains("import "));
        assert_eq!(
            output,
            render_certified_package(&JavaStructuralRenderer, api.package()).unwrap()
        );
    }
}
#[test]
fn constant_identity_is_certificate_scoped_and_survives_owner_drop() {
    let first = c::api(false);
    let second = c::api(false);
    let constant = first.constants().next().unwrap().clone();
    assert_eq!(&constant, first.constants().next().unwrap());
    assert_ne!(&constant, second.constants().next().unwrap());
    let generated = constant.generated();
    drop(first);
    assert!(
        constant.package_identity().package().ast().files()[0].items()[0]
            .source_inventory
            .get(GeneratedSymbolId::Value(generated))
            .is_some()
    );
}
#[test]
fn constant_shape_and_initializer_controls_are_rejected() {
    for fault in 0..10 {
        let mut fixture = c::Fixture::new(true);
        match fault {
            0 => fixture.field(0).declared = None,
            1 => fixture
                .field(0)
                .modifiers
                .retain(|m| *m != JavaModifier::Final),
            2 => fixture
                .field(0)
                .modifiers
                .retain(|m| *m != JavaModifier::Static),
            3 => fixture.field(0).modifiers[0] = JavaModifier::Private,
            4 => fixture.field(0).initializer = None,
            5 => fixture.field(0).ty = f::int(),
            6 => {
                fixture.field(0).initializer =
                    Some(JavaExpr::literal(f::boolean(), JavaLiteral::I32(1)))
            }
            7 => {
                fixture.field(0).initializer =
                    Some(JavaExpr::local(f::boolean(), f::name("missing")))
            }
            8 => fixture.field(0).declared = fixture.field(1).declared,
            9 => {
                fixture.facade.members.remove(1);
            }
            _ => unreachable!(),
        }
        assert!(c::admit(fixture.finish()).is_err(), "fault {fault}");
    }
}
#[test]
fn constant_registrations_cannot_forge_source_authority() {
    for fault in 0..10 {
        let fixture = c::Fixture::configured(
            false,
            |_| {},
            |i, registration| {
                if i != 0 {
                    return;
                }
                match fault {
                    0 => registration.name = "wrong".into(),
                    1 => registration.visibility = JavaVisibility::Private,
                    2 => registration.ty = JavaDialect.registered_type(&f::int()),
                    3 => {
                        registration.origin =
                            GeneratedOrigin::Synthesized(SynthesisReason::TestHarness)
                    }
                    _ => {
                        let GeneratedOrigin::RustSource(source) = &mut registration.origin else {
                            panic!()
                        };
                        let source = std::sync::Arc::make_mut(source);
                        match fault {
                            4 => source.externally_reachable = false,
                            5 => source.visibility = RustVisibility::RestrictedTo(source.module),
                            6 => source.declaration.crate_id += 1,
                            7 => source.declaration.definition_path_hash = 11,
                            9 => source.node = RustSourceNode::Binding(0),
                            8 => {
                                source.crate_exports = std::sync::Arc::new(RustCrateExports {
                                    root: source.crate_exports.root,
                                    modules: Default::default(),
                                    module_ancestries: Default::default(),
                                })
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            },
        );
        assert!(
            c::admit(fixture.finish()).is_err(),
            "registration fault {fault}"
        );
    }
}
#[test]
fn complete_constant_exports_reject_missing_extra_and_wrong_namespace() {
    for fault in 0..3 {
        let fixture = c::Fixture::with_exports(false, |exports| {
            let bindings = exports.modules.get_mut(&exports.root).unwrap();
            match fault {
                0 => bindings
                    .retain(|_, value| *value != RustExportTarget::Declaration(f::id(0x35c, 10))),
                1 => {
                    bindings.insert(
                        RustExportName {
                            namespace: RustExportNamespace::Value,
                            name: "extra".into(),
                        },
                        RustExportTarget::Declaration(f::id(0x35c, 999)),
                    );
                }
                2 => {
                    bindings.insert(
                        RustExportName {
                            namespace: RustExportNamespace::Type,
                            name: "wrong".into(),
                        },
                        RustExportTarget::Declaration(f::id(0x35c, 10)),
                    );
                }
                _ => unreachable!(),
            }
        });
        assert!(c::admit(fixture.finish()).is_err(), "export fault {fault}");
    }
}

#[test]
fn generated_constant_reads_do_not_enable_assignment() {
    let mut fixture = c::Fixture::new(true);
    let method = fixture
        .facade
        .members
        .iter_mut()
        .find_map(|member| match member {
            JavaMember::Method(method) => Some(method),
            _ => None,
        })
        .unwrap();
    let body = method.body.as_mut().unwrap();
    let JavaStmt::Return(Some(target)) = &body.statements[0] else {
        panic!()
    };
    body.statements.insert(
        0,
        JavaStmt::Assign {
            target: target.clone(),
            value: JavaExpr::literal(f::boolean(), JavaLiteral::Boolean(true)),
        },
    );
    assert!(c::admit(fixture.finish()).is_err());
}
