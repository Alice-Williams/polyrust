//! Export-only roots must retain original certificates without synthetic definitions.
use super::{constant_exports_fixture as f, source_constant_fixture as c};
use crate::{ast::*, dialect::*};
use portable_codegen::*;

#[test]
fn alias_only_and_mixed_views_keep_original_authority_and_qualified_roots() {
    let producer = c::api(false);
    let values: Vec<_> = producer.constants().cloned().collect();
    let owner = producer.package_identity().clone();
    for own in [false, true] {
        let api = c::admit(f::Fixture::new(0x600, &values, own).finish()).unwrap();
        assert_eq!(api.constants().len(), usize::from(own));
        assert_eq!(api.functions().len(), 0);
        assert_eq!(api.foreign_constants().len(), 16);
        assert_eq!(api.dependencies().len(), 1);
        assert_eq!(api.source_descriptions().unwrap().len(), usize::from(own));
        let item = &api.package().ast().files()[0].items()[0];
        let mut seen = std::collections::BTreeSet::new();
        for export in api.foreign_constants() {
            assert!(seen.insert((export.module(), export.name().clone())));
            assert_eq!(export.dependency().package_identity(), &owner);
            assert!(values.contains(export.dependency()));
        }
        assert_eq!(
            item.names
                .keys()
                .filter(|key| matches!(key, TargetSymbolRef::DependencyValue(_)))
                .count(),
            8
        );
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        let OutputContents::Text(text) = output.files()[0].contents() else {
            panic!()
        };
        assert!(api.source_byte_bound().unwrap() >= text.len() as u64);
        assert!(
            !text.contains("import ") && !text.contains("Runtime") && !text.contains("renamed")
        );
        assert_eq!(
            text.matches("public static final").count(),
            usize::from(own)
        );
        verify_linked_package(api.package().ast()).unwrap();
    }
    let middle = c::admit(f::Fixture::new(0x601, &values, false).finish()).unwrap();
    let original = middle
        .foreign_constants()
        .next()
        .unwrap()
        .dependency()
        .clone();
    drop(middle);
    drop(producer);
    let root =
        c::admit(f::Fixture::new(0x602, std::slice::from_ref(&original), false).finish()).unwrap();
    assert_eq!(
        root.foreign_constants().next().unwrap().dependency(),
        &original
    );
    assert_eq!(root.dependencies().next().unwrap(), &owner);
}

#[test]
fn absent_wrong_kind_and_conflicting_export_witnesses_reject() {
    let producer = c::api(false);
    let value = producer.constants().next().unwrap().clone();
    for fault in 0..7 {
        let mut fixture = f::Fixture::new(0x600, std::slice::from_ref(&value), false);
        let root = fixture.graph.root;
        match fault {
            0 => fixture.dependencies = JavaDependencyBindings::default(),
            1 => {
                fixture.dependencies = f::imports([producer.constants().nth(1).unwrap().clone()]);
            }
            2 => {
                let names = fixture.graph.modules.get_mut(&root).unwrap();
                let target = names.remove(&f::binding("renamed0")).unwrap();
                names.insert(
                    RustExportName {
                        namespace: RustExportNamespace::Type,
                        name: "wrong".into(),
                    },
                    target,
                );
            }
            3 => {
                fixture.graph.modules.get_mut(&root).unwrap().insert(
                    f::binding("renamed0"),
                    RustExportTarget::Module(value.declaration()),
                );
            }
            4 => {
                let function = super::source_dependency_fixture::package(
                    0x999,
                    super::source_dependency_fixture::functions(42),
                );
                let function = c::admit(function)
                    .unwrap()
                    .functions()
                    .next()
                    .unwrap()
                    .clone();
                fixture.graph.modules.get_mut(&root).unwrap().insert(
                    f::binding("function"),
                    RustExportTarget::Declaration(function.declaration()),
                );
                fixture.dependencies = JavaDependencyScope::new()
                    .import(function)
                    .0
                    .import_constant(value.clone())
                    .0
                    .finish();
            }
            5 => {
                let independent = c::api(false);
                fixture.dependencies = f::imports([
                    value.clone(),
                    independent.constants().next().unwrap().clone(),
                ]);
            }
            6 => {
                fixture.graph.modules.get_mut(&root).unwrap().insert(
                    RustExportName {
                        namespace: RustExportNamespace::Macro,
                        name: "macro_alias".into(),
                    },
                    RustExportTarget::Declaration(value.declaration()),
                );
            }
            _ => unreachable!(),
        }
        assert!(c::admit(fixture.finish()).is_err(), "fault {fault}");
    }
    assert!(c::admit(f::Fixture::new(0x600, &[], false).finish()).is_err());
}

#[test]
fn zero_owned_closure_rejects_transitive_self_and_independent_certificate_conflicts() {
    let producer = c::api(false);
    let values: Vec<_> = producer.constants().cloned().collect();
    let middle = c::admit(f::Fixture::new(0x610, &values, true).finish()).unwrap();
    let middle_value = middle.constants().next().unwrap().clone();
    // Selected constant belongs to middle, but its complete certificate depends on producer.
    let positive =
        c::admit(f::Fixture::new(0x611, std::slice::from_ref(&middle_value), false).finish())
            .unwrap();
    assert_eq!(positive.constants().len(), 0);
    assert_eq!(
        positive.dependencies().next().unwrap(),
        middle.package_identity()
    );
    assert!(
        c::admit(f::Fixture::new(0x35c, std::slice::from_ref(&middle_value), false).finish())
            .is_err()
    );
    assert!(
        c::admit(f::Fixture::new(0x610, std::slice::from_ref(&middle_value), false).finish())
            .is_err()
    );
    let independent = c::api(false);
    let mut fixture = f::Fixture::new(0x612, std::slice::from_ref(&middle_value), false);
    fixture.dependencies = f::imports([
        middle_value.clone(),
        independent.constants().next().unwrap().clone(),
    ]);
    assert!(c::admit(fixture.finish()).is_err());
    let mut diamond = f::Fixture::new(0x613, std::slice::from_ref(&middle_value), false);
    diamond.dependencies = f::imports([middle_value, values[0].clone()]);
    c::admit(diamond.finish()).unwrap();
}

#[test]
fn export_only_resolved_paths_scopes_and_projection_changes_cannot_replace_original() {
    let producer = c::api(false);
    let values: Vec<_> = producer.constants().cloned().collect();
    let api = c::admit(f::Fixture::new(0x600, &values, false).finish()).unwrap();
    let expected = &api.package().ast().files()[0].items()[0];
    let key = expected
        .names
        .keys()
        .find(|key| matches!(key, TargetSymbolRef::DependencyValue(_)))
        .unwrap()
        .clone();
    for fault in 0..5 {
        let mut changed = expected.clone();
        match fault {
            0 => {
                changed.names.remove(&key);
            }
            1 => {
                changed.names.insert(
                    key.clone(),
                    JavaResolvedName::Local(super::source_dependency_fixture::name("forged")),
                );
            }
            2 | 3 => {
                let JavaFileItem::Type { dependencies, .. } = &mut changed.item else {
                    panic!()
                };
                *dependencies = f::imports(values.clone()); // Independent consumer scope.
                if fault == 3 {
                    changed.names.remove(&key);
                }
            }
            4 => {
                let keys: Vec<_> = changed
                    .names
                    .keys()
                    .filter(|key| matches!(key, TargetSymbolRef::DependencyValue(_)))
                    .cloned()
                    .collect();
                let left = changed.names[&keys[0]].clone();
                let right = changed.names[&keys[1]].clone();
                changed.names.insert(keys[0].clone(), right);
                changed.names.insert(keys[1].clone(), left);
            }
            _ => unreachable!(),
        }
        assert_ne!(&changed, expected);
        assert!(
            !JavaDialect.verify_resolved_file_item(&changed).is_empty(),
            "projection {fault}"
        );
    }
}
