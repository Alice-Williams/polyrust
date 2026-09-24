//! Target foundation: exact standard fields without source capability admission.
use super::{
    constant_exports_fixture as aliases, source_constant_consumer_fixture as imports,
    source_constant_fixture as c, source_dependency_fixture as f,
};
use crate::{ast::*, dialect::*};
use portable_binary64::Binary64Sign;
use portable_codegen::*;

#[path = "infinite_constant_native.rs"]
mod native;
#[path = "infinite_constant_rejections.rs"]
mod rejections;

pub(crate) fn expression(sign: Binary64Sign) -> JavaExpr {
    JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Double),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::KnownField(match sign {
            Binary64Sign::Positive => JavaKnownField::DoublePositiveInfinity,
            Binary64Sign::Negative => JavaKnownField::DoubleNegativeInfinity,
        })),
    }
}
pub(crate) fn fixture(mixed: bool) -> c::Fixture {
    let mut source = super::finite_constants::fixture(&[0, 0], mixed);
    for (index, sign) in [Binary64Sign::Positive, Binary64Sign::Negative]
        .into_iter()
        .enumerate()
    {
        source.field(index).initializer = Some(expression(sign));
    }
    source
}
fn packages() -> Vec<JavaDependencyApi> {
    let owner = c::admit(fixture(true).finish()).unwrap();
    let values: Vec<_> = owner.constants().cloned().collect();
    let facade = c::admit(aliases::Fixture::new(0x925, &values, false).finish()).unwrap();
    let selected: Vec<_> = facade
        .foreign_constants()
        .filter(|v| v.module() == facade.root())
        .map(|v| v.dependency().clone())
        .collect();
    assert_eq!(selected, values);
    assert_eq!(facade.constants().len(), 0);
    assert_eq!(facade.functions().len(), 0);
    let mut scope = JavaDependencyScope::new();
    let mut reads = vec![];
    for value in selected {
        let (next, read) = scope.import_constant(value).unwrap();
        scope = next;
        reads.push(read);
    }
    let reader = c::admit(imports::consumer(0x926, scope.finish(), &reads, None)).unwrap();
    vec![owner, facade, reader]
}
#[cfg(test)]
fn text(api: &JavaDependencyApi) -> String {
    let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
    assert_eq!(output.files().len(), 1);
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("source")
    };
    assert!(text.len() as u64 <= api.source_byte_bound().unwrap());
    assert!(
        !text.contains("Runtime") && !text.contains("import ") && !text.contains("doubleValue")
    );
    text.clone()
}
#[test]
fn infinities_keep_exact_primitive_values_descriptions_dependencies_and_alias_authority() {
    let packages = packages();
    let owner = &packages[0];
    for (index, sign) in [Binary64Sign::Positive, Binary64Sign::Negative]
        .into_iter()
        .enumerate()
    {
        let constant = owner.constant(f::id(0x35c, 10 + index as u64)).unwrap();
        assert_eq!(constant.value(), &JavaScalarConstantValue::Infinity(sign));
        assert_eq!(constant.value().literal(), None);
        assert_eq!(constant.ty(), &JavaType::primitive(JavaPrimitive::Double));
        assert_eq!(constant.package_identity(), owner.package_identity());
        assert_eq!(
            constant.source().documentation,
            [format!(" Constant {index} documentation.")]
        );
        let descriptions = owner.source_descriptions().unwrap();
        let description = descriptions
            .iter()
            .find(|d| d.source().declaration == constant.declaration())
            .unwrap();
        assert_eq!(
            description.kind(),
            JavaSourceDescriptionKind::Constant {
                ty: constant.ty(),
                value: constant.value()
            }
        );
    }
    for api in &packages {
        text(api);
    }
    assert_eq!(
        packages[2].dependencies().next().unwrap(),
        owner.package_identity()
    );
    let output = text(owner);
    assert!(
        output.contains("Double.POSITIVE_INFINITY") && output.contains("Double.NEGATIVE_INFINITY")
    );
    assert!(!text(&packages[1]).contains("double"));
    for field in [
        JavaKnownField::DoublePositiveInfinity,
        JavaKnownField::DoubleNegativeInfinity,
    ] {
        assert_eq!(field.owner(), JavaKnownType::Double);
        assert_eq!(field.ty(), JavaType::primitive(JavaPrimitive::Double));
        let mut symbols = std::collections::BTreeSet::new();
        expression(if field == JavaKnownField::DoublePositiveInfinity {
            Binary64Sign::Positive
        } else {
            Binary64Sign::Negative
        })
        .symbols(&mut symbols);
        assert!(symbols.contains(&TargetSymbolRef::KnownType(JavaKnownType::Double.into())));
        assert!(symbols.contains(&TargetSymbolRef::KnownField(field)));
    }
}
#[test]
fn infinity_lookalike_producers_never_replace_import_authority() {
    let owner = c::admit(fixture(false).finish()).unwrap();
    let mut changed = fixture(false);
    changed.field(0).initializer = Some(expression(Binary64Sign::Negative));
    for other in [
        c::admit(fixture(false).finish()).unwrap(),
        c::admit(changed.finish()).unwrap(),
    ] {
        let before = owner.constants().next().unwrap();
        let after = other.constants().next().unwrap();
        assert_ne!(before, after);
        let (scope, _) = JavaDependencyScope::new()
            .import_constant(after.clone())
            .unwrap();
        let (_, read) = JavaDependencyScope::new()
            .import_constant(before.clone())
            .unwrap();
        assert!(c::admit(imports::consumer(0x926, scope.finish(), &[read], None)).is_err());
    }
}
