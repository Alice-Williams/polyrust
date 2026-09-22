//! Exact constant certificates and original import authority; not source admission.
use super::{
    source_constant_consumer_fixture as imports, source_constant_fixture as c,
    source_dependency_fixture as f,
};
use crate::{ast::*, dialect::*};
use portable_binary64::FiniteBinary64;
use portable_codegen::*;

#[path = "finite_constant_native.rs"]
mod native;
#[path = "finite_constant_rejections.rs"]
mod rejections;

pub(crate) fn literal(bits: u64) -> JavaLiteral {
    JavaLiteral::F64(FiniteBinary64::from_bits(bits).unwrap())
}

pub(crate) fn fixture(bits: &[u64], mixed: bool) -> c::Fixture {
    c::Fixture::with_values(mixed, bits.iter().map(|bits| literal(*bits)).collect())
}

fn api(bits: &[u64], mixed: bool) -> JavaDependencyApi {
    c::admit(fixture(bits, mixed).finish()).unwrap()
}

fn consumer(owner: &JavaDependencyApi) -> (TargetAstPackage<JavaDialect>, Vec<JavaImportedValue>) {
    let mut scope = JavaDependencyScope::new();
    let mut values = vec![];
    for constant in owner.constants() {
        let (next, imported) = scope.import_constant(constant.clone());
        scope = next;
        values.push(imported);
    }
    (
        imports::consumer(0x924, scope.finish(), &values, None),
        values,
    )
}

fn check_bounds(owner: &JavaDependencyApi) {
    let output = render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
    let mut bytes = 0;
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("source")
        };
        bytes += text.len() as u64;
        assert!(!text.contains("Runtime") && !text.contains("doubleValue"));
    }
    assert!(bytes <= owner.source_byte_bound().unwrap());
}

#[test]
fn finite_constants_keep_exact_bits_types_docs_and_imported_readers() {
    let bits = [0, 1 << 63, 1, 0x0010_0000_0000_0000, 0x7fef_ffff_ffff_ffff];
    for mixed in [false, true] {
        let owner = api(&bits, mixed);
        assert_eq!(owner.constants().len(), bits.len());
        assert_eq!(owner.functions().len(), if mixed { bits.len() } else { 0 });
        for (index, value) in bits.iter().enumerate() {
            let constant = owner.constant(f::id(0x35c, 10 + index as u64)).unwrap();
            assert_eq!(constant.value(), &literal(*value));
            assert_eq!(constant.ty(), &JavaType::primitive(JavaPrimitive::Double));
            assert_eq!(
                constant.source().documentation,
                [format!(" Constant {index} documentation.")]
            );
            assert_eq!(constant.package_identity(), owner.package_identity());
        }
        check_bounds(&owner);
        let (draft, values) = consumer(&owner);
        let reader = c::admit(draft).unwrap();
        assert_eq!(reader.functions().len(), bits.len());
        assert_eq!(reader.constants().len(), 0);
        assert_eq!(
            reader.dependencies().next().unwrap(),
            owner.package_identity()
        );
        assert!(
            values
                .iter()
                .all(|value| value.ty() == &JavaType::primitive(JavaPrimitive::Double))
        );
        check_bounds(&reader);
    }
}

#[test]
fn signed_zero_and_lookalike_producers_cannot_replace_exact_authority() {
    let owner = api(&[0], false);
    let (draft, values) = consumer(&owner);
    let catalogue = JavaDialect.package_symbol_catalogue(&draft).unwrap();
    for bits in [0, 1 << 63] {
        let other = api(&[bits], false);
        let before = owner.constants().next().unwrap();
        let after = other.constants().next().unwrap();
        assert_eq!(before.value() == after.value(), bits == 0);
        assert_ne!(before, after);
        assert_ne!(before.package_identity(), after.package_identity());
        let mut changed = catalogue.clone();
        changed.dependency_values[0].owner = other.package_identity().clone();
        assert!(changed.verify(&JavaDialect).is_err());
        let (foreign_scope, _) = JavaDependencyScope::new().import_constant(after.clone());
        assert!(
            c::admit(imports::consumer(
                0x924,
                foreign_scope.finish(),
                &values,
                None
            ))
            .is_err()
        );
    }
}
