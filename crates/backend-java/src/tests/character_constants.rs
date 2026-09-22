//! Target Int certificates are not source Char witnesses.
use super::{
    constant_exports_fixture as aliases, source_constant_consumer_fixture as imports,
    source_constant_fixture as c, source_dependency_fixture as f,
};
use crate::{ast::*, dialect::*};
use portable_codegen::*;

#[path = "character_constant_native.rs"]
mod native;
#[path = "character_constant_rejections.rs"]
mod rejections;

fn fixture(values: &[i32], readers: bool) -> c::Fixture {
    assert!(values.len() <= 64);
    c::Fixture::with_values(
        readers,
        values.iter().copied().map(JavaLiteral::I32).collect(),
    )
}

fn packages(values: &[i32]) -> Vec<JavaDependencyApi> {
    let owner = c::admit(fixture(values, true).finish()).unwrap();
    let constants: Vec<_> = owner.constants().cloned().collect();
    let facade = c::admit(aliases::Fixture::new(0x925, &constants, false).finish()).unwrap();
    assert_eq!(facade.constants().len(), 0);
    assert_eq!(facade.functions().len(), 0);
    assert_eq!(facade.foreign_constants().len(), 2 * values.len());
    let mut scope = JavaDependencyScope::new();
    let mut reads = vec![];
    for constant in &constants {
        let export = facade
            .foreign_constants()
            .find(|export| {
                export.module() == facade.root()
                    && export.dependency().declaration() == constant.declaration()
            })
            .unwrap();
        assert_eq!(export.dependency(), constant);
        let (next, read) = scope.import_constant(export.dependency().clone());
        scope = next;
        reads.push(read);
    }
    let reader = c::admit(imports::consumer(0x926, scope.finish(), &reads, None)).unwrap();
    assert_eq!(
        reader.dependencies().next().unwrap(),
        owner.package_identity()
    );
    vec![owner, facade, reader]
}

fn text(api: &JavaDependencyApi) -> String {
    let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
    assert_eq!(output.files().len(), 1);
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("source")
    };
    assert!(text.len() as u64 <= api.source_byte_bound().unwrap());
    for forbidden in ["Runtime", "import ", "Character", "Integer", "char "] {
        assert!(!text.contains(forbidden), "{forbidden}");
    }
    text.clone()
}

#[test]
fn scalar_and_target_only_int_constants_retain_exact_types_values_and_original_aliases() {
    let values = [
        0,
        0xd7ff,
        0xd800,
        0xdfff,
        0xe000,
        0xffff,
        0x10000,
        0x10ffff,
        0x110000,
        -1,
        i32::MIN,
        i32::MAX,
    ];
    for readers in [false, true] {
        let owner = c::admit(fixture(&values, readers).finish()).unwrap();
        assert_eq!(owner.constants().len(), values.len());
        assert_eq!(
            owner.functions().len(),
            if readers { values.len() } else { 0 }
        );
        for (index, value) in values.iter().enumerate() {
            let constant = owner.constant(f::id(0x35c, 10 + index as u64)).unwrap();
            assert_eq!(constant.value(), &JavaScalarConstantValue::I32(*value));
            assert_eq!(constant.value().literal(), Some(JavaLiteral::I32(*value)));
            assert_eq!(constant.ty(), &f::int());
            assert_eq!(constant.package_identity(), owner.package_identity());
            assert_eq!(
                constant.source().documentation,
                [format!(" Constant {index} documentation.")]
            );
            let descriptions = owner.source_descriptions().unwrap();
            let description = descriptions
                .iter()
                .find(|item| item.source().declaration == constant.declaration())
                .unwrap();
            assert_eq!(
                description.kind(),
                JavaSourceDescriptionKind::Constant {
                    ty: constant.ty(),
                    value: constant.value(),
                }
            );
        }
        text(&owner);
    }
    for api in packages(&values) {
        text(&api);
    }
}

#[test]
fn int_constant_lookalikes_cannot_replace_exact_import_authority() {
    let owner = c::admit(fixture(&[0x10ffff], false).finish()).unwrap();
    let before = owner.constants().next().unwrap();
    for value in [0x10ffff, 0x10fffe] {
        let other = c::admit(fixture(&[value], false).finish()).unwrap();
        let after = other.constants().next().unwrap();
        assert_eq!(before.value() == after.value(), value == 0x10ffff);
        assert_ne!(before, after);
        assert_ne!(before.package_identity(), after.package_identity());
        let (scope, read) = JavaDependencyScope::new().import_constant(before.clone());
        let draft = imports::consumer(0x926, scope.finish(), std::slice::from_ref(&read), None);
        let mut catalogue = JavaDialect.package_symbol_catalogue(&draft).unwrap();
        catalogue.dependency_values[0].owner = other.package_identity().clone();
        assert!(catalogue.verify(&JavaDialect).is_err());
        let (foreign, _) = JavaDependencyScope::new().import_constant(after.clone());
        assert!(c::admit(imports::consumer(0x926, foreign.finish(), &[read], None)).is_err());
    }
}
