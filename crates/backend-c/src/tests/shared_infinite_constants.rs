//! Exact infinity syntax, original authority and derived imports/resources.
use super::{
    CDependencyApi, CDialect, CStructuralRenderer, constant_consumer_fixture,
    constant_export_fixture,
    constant_producer_tests::certify,
    owned_constant_fixture::{self, Fixture},
    owned_constant_tests::linked,
    project_c_package,
};
use crate::ast::*;
use portable_binary64::{Binary64Sign, FiniteBinary64};
use portable_codegen::*;

#[path = "shared_infinite_constant_native.rs"]
mod native;
#[path = "shared_infinite_constant_rejections.rs"]
mod rejections;

fn expression(ast: &CExpressions<'_>, sign: Binary64Sign) -> CValue {
    let value = ast.known_constant(CKnownConstant::DoubleInfinity);
    match sign {
        Binary64Sign::Positive => value,
        Binary64Sign::Negative => ast.unary(CUnaryOperator::Negate, value).unwrap(),
    }
}

fn set_values(source: &mut Fixture, signs: &[Binary64Sign]) {
    let ast = CExpressions::new(source.registry.registrations());
    let declarations = CDeclarations::new(
        source.registry.registrations(),
        source.files[1].identity().clone(),
    )
    .unwrap();
    source.files[1] = declarations
        .source_file(
            source
                .objects
                .iter()
                .zip(signs)
                .map(|(object, sign)| {
                    CFileItem::Definition(
                        declarations
                            .object_definition(
                                object.clone(),
                                CLinkage::External,
                                ast.expression_initializer(expression(&ast, *sign)).unwrap(),
                            )
                            .unwrap(),
                    )
                })
                .collect(),
        )
        .unwrap();
}

fn fixture(signs: &[Binary64Sign]) -> Fixture {
    let names: Vec<_> = (0..signs.len())
        .map(|index| format!("infinite_{index}"))
        .collect();
    let values: Vec<_> = names
        .iter()
        .map(|name| {
            (
                name.as_str(),
                CLiteral::F64(FiniteBinary64::from_bits(0).unwrap()),
            )
        })
        .collect();
    let mut source = owned_constant_fixture::literal_fixture(&values);
    set_values(&mut source, signs);
    source
}

fn api(source: &Fixture) -> CDependencyApi {
    CDependencyApi::from_certificate(certify(source)).unwrap()
}

#[cfg(test)]
fn check_resources(owner: &CDependencyApi, expect_math: bool) {
    let output = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
    assert_eq!(output.files().len(), 2);
    let mut bytes = 0;
    let mut math_headers = 0;
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("C text")
        };
        bytes += text.len() as u64;
        math_headers += text.matches("#include <math.h>").count();
        assert!(!text.contains("runtime") && !text.contains("goto "));
    }
    assert_eq!(math_headers, usize::from(expect_math));
    assert!(bytes <= crate::dialect::c_output_byte_bound(owner.package()).unwrap());
    assert!(owner.system_libraries().is_empty());
}

#[test]
fn signed_infinities_retain_exact_values_original_alias_authority_and_bounds() {
    let signs = [Binary64Sign::Positive, Binary64Sign::Negative];
    let source = fixture(&signs);
    let original = api(&source);
    for (constant, sign) in original.constants().zip(signs) {
        assert_eq!(constant.value(), &CScalarConstantValue::Infinity(sign));
        assert_eq!(constant.value().literal(), None);
        assert_eq!(constant.read_type(), &CObjectType::scalar(CScalarType::F64));
        assert_eq!(constant.object().ty().constness(), CConstness::Const);
        assert_eq!(constant.package_identity().stack_bound_bytes(), 0);
    }
    check_resources(&original, true);
    let values: Vec<_> = original.constants().cloned().collect();
    let facade = api(&constant_export_fixture::facade(921, &values));
    let aliases: Vec<_> = facade
        .foreign_constants()
        .map(|binding| binding.dependency().clone())
        .collect();
    assert_eq!(aliases, values);
    check_resources(&facade, false);
    let reader = api(&constant_consumer_fixture::fixture(
        922,
        &aliases,
        None,
        constant_consumer_fixture::Usage::Read,
    ));
    assert_eq!(reader.functions().count(), signs.len());
    check_resources(&reader, false);
    let measured = super::resources::measure_package(&linked(&source)).unwrap();
    assert_eq!(measured.total.frame_bound, 0);
    assert!(measured.total.value_bytes >= 16);
}

#[test]
fn infinity_signs_and_identical_values_do_not_conflate_owner_certificates() {
    let mut source = fixture(&[Binary64Sign::Positive]);
    let original = api(&source);
    set_values(&mut source, &[Binary64Sign::Negative]);
    let changed = api(&source);
    let lookalike = api(&fixture(&[Binary64Sign::Positive]));
    let before = original.constants().next().unwrap();
    let after = changed.constants().next().unwrap();
    assert_eq!(before.object(), after.object());
    assert_eq!(before.declaration(), after.declaration());
    assert_ne!(before.value(), after.value());
    assert_eq!(
        before.value(),
        lookalike.constants().next().unwrap().value()
    );
    for replacement in [after, lookalike.constants().next().unwrap()] {
        assert_ne!(before, replacement);
        assert_ne!(before.cmp(replacement), std::cmp::Ordering::Equal);
        assert_ne!(before.package_identity(), replacement.package_identity());
        let mut registry = CRegistry::new();
        registry.import_constant(before.clone()).unwrap();
        assert!(registry.import_constant(replacement.clone()).is_err());
    }
}
