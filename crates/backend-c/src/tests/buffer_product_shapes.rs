//! Exact count identities, immutable product chains and layout matching.
use super::{product_fixture::*, *};
use crate::ast::{numeric_fixture::Fixture, *};

#[derive(Clone, Copy, Debug)]
enum Form {
    Direct,
    Reversed,
    Converted,
    Materialized,
    Chained,
}

#[test]
fn direct_reversed_and_immutable_products_match_measured_element_layouts() {
    for (element, stride) in [
        (CObjectType::scalar(CScalarType::U8), 1),
        (CObjectType::scalar(CScalarType::Int), 4),
        (CObjectType::scalar(CScalarType::I64), 8),
        (
            CObjectType::array(
                CObjectType::scalar(CScalarType::Int),
                CArrayLength::new(3).unwrap(),
            )
            .unwrap(),
            12,
        ),
    ] {
        for form in [
            Form::Direct,
            Form::Reversed,
            Form::Converted,
            Form::Materialized,
            Form::Chained,
        ] {
            let mut f = Fixture::new(&[]);
            let count = count(&mut f, "count");
            let buffer = buffer(&mut f, "buffer", &count, element.clone());
            let mut body = vec![f.declare(count.local(), f.size(2))];
            let size = f.values().size_of(element.clone()).unwrap();
            let product = match form {
                Form::Direct if stride == 1 => f.read(count.local()),
                Form::Reversed => multiply(&f, size, f.read(count.local())),
                _ => multiply(&f, f.read(count.local()), size),
            };
            let bytes = match form {
                Form::Converted => f
                    .values()
                    .numeric_conversion(
                        CScalarType::Size,
                        f.values()
                            .numeric_conversion(CScalarType::U64, product)
                            .unwrap(),
                    )
                    .unwrap(),
                Form::Materialized | Form::Chained => {
                    let local = immutable(&mut f, "bytes", CScalarType::Size);
                    body.push(f.declare(&local, product));
                    if matches!(form, Form::Chained) {
                        let alias = immutable(&mut f, "alias", CScalarType::U64);
                        body.push(
                            f.declare(
                                &alias,
                                f.values()
                                    .numeric_conversion(CScalarType::U64, f.read(&local))
                                    .unwrap(),
                            ),
                        );
                        f.values()
                            .numeric_conversion(CScalarType::Size, f.read(&alias))
                            .unwrap()
                    } else {
                        f.read(&local)
                    }
                }
                _ => product,
            };
            body.push(allocate(&f, bytes));
            let actual = requests(&f, body).unwrap().remove(0);
            assert_eq!(actual.bytes, (2 * stride, 2 * stride), "{form:?}");
            assert_eq!(
                actual.element_bounds(&buffer, &f.registry),
                Ok((2, 2)),
                "{form:?}"
            );
            assert_eq!(
                actual.admits_object(&buffer, &f.registry),
                Err(E::UnprovedAllocation)
            );
        }
    }
}

#[test]
fn checked_immutable_factors_and_nested_products_retain_the_original_count() {
    let mut f = Fixture::new(&[]);
    let count = count(&mut f, "count");
    let factor = immutable(&mut f, "factor", CScalarType::Size);
    let alias = immutable(&mut f, "factor_alias", CScalarType::U64);
    let buffer = buffer(
        &mut f,
        "buffer",
        &count,
        CObjectType::scalar(CScalarType::I64),
    );
    let product = multiply(
        &f,
        f.size(2),
        multiply(
            &f,
            f.read(count.local()),
            f.values()
                .numeric_conversion(CScalarType::Size, f.read(&alias))
                .unwrap(),
        ),
    );
    let actual = requests(
        &f,
        vec![
            f.declare(count.local(), f.size(3)),
            f.declare(&factor, f.size(4)),
            f.declare(
                &alias,
                f.values()
                    .numeric_conversion(CScalarType::U64, f.read(&factor))
                    .unwrap(),
            ),
            allocate(&f, product),
        ],
    )
    .unwrap()
    .remove(0);
    assert_eq!(actual.element_bounds(&buffer, &f.registry), Ok((3, 3)));
}

#[test]
fn equal_values_wrong_stride_and_insufficient_alignment_cannot_rebind_a_product() {
    let mut f = Fixture::new(&[]);
    let original = count(&mut f, "count");
    let other = count(&mut f, "other_count");
    let valid = buffer(
        &mut f,
        "valid",
        &original,
        CObjectType::scalar(CScalarType::I64),
    );
    let wrong_count = buffer(
        &mut f,
        "wrong_count",
        &other,
        CObjectType::scalar(CScalarType::I64),
    );
    let wrong_stride = buffer(
        &mut f,
        "wrong_stride",
        &original,
        CObjectType::scalar(CScalarType::Int),
    );
    let actual = requests(
        &f,
        vec![
            f.declare(original.local(), f.size(2)),
            f.declare(other.local(), f.size(2)),
            allocate(&f, multiply(&f, f.read(original.local()), f.size(8))),
        ],
    )
    .unwrap()
    .remove(0);
    assert_eq!(actual.element_bounds(&valid, &f.registry), Ok((2, 2)));
    for descriptor in [&wrong_count, &wrong_stride] {
        assert_eq!(
            actual.element_bounds(descriptor, &f.registry),
            Err(E::UnprovedAllocationSize)
        );
    }
    let mut weak_alignment = actual;
    weak_alignment.alignment = 4;
    assert_eq!(
        weak_alignment.element_bounds(&valid, &f.registry),
        Err(E::UnprovedAllocationSize)
    );
}

#[test]
fn mutable_byte_initializers_and_unknown_algebra_are_not_count_evidence() {
    for form in 0..4 {
        let mut f = Fixture::new(&[]);
        let count = count(&mut f, "count");
        let buffer = buffer(
            &mut f,
            "buffer",
            &count,
            CObjectType::scalar(CScalarType::I64),
        );
        let mut body = vec![f.declare(count.local(), f.size(2))];
        let product = multiply(&f, f.read(count.local()), f.size(8));
        let bytes = match form {
            0 | 1 => {
                let local = f.local(CScalarType::Size, "bytes");
                body.push(f.declare(&local, product));
                if form == 1 {
                    body.push(
                        f.ast()
                            .assign(f.values().local(local.clone()).unwrap(), f.size(16))
                            .unwrap(),
                    );
                }
                f.read(&local)
            }
            2 => f.binary(CBinaryOperator::Add, product, f.size(0)),
            _ => f.size(16),
        };
        body.push(allocate(&f, bytes));
        let actual = requests(&f, body).unwrap().remove(0);
        assert_eq!(actual.bytes, (16, 16));
        assert_eq!(
            actual.element_bounds(&buffer, &f.registry),
            Err(E::UnprovedAllocationSize)
        );
    }
}

#[test]
fn product_joins_keep_only_shared_identities_and_union_original_count_bounds() {
    let mut f = Fixture::new(&[CScalarType::Size]);
    let count = count(&mut f, "count");
    let descriptor = buffer(
        &mut f,
        "buffer",
        &count,
        CObjectType::scalar(CScalarType::I64),
    );
    let low = f.compare(CBinaryOperator::Greater, f.input(0), f.size(0));
    let high = f.compare(CBinaryOperator::LessEqual, f.input(0), f.size(3));
    let low = assume(&mut f, low);
    let high = assume(&mut f, high);
    let actual = requests(
        &f,
        vec![
            low,
            high,
            f.declare(count.local(), f.input(0)),
            allocate(&f, multiply(&f, f.read(count.local()), f.size(8))),
        ],
    )
    .unwrap()
    .remove(0);
    let mut narrowed = actual.clone();
    narrowed.bytes = (8, 16);
    assert_eq!(
        narrowed.element_bounds(&descriptor, &f.registry),
        Ok((1, 2))
    );
    assert_eq!(
        actual
            .join(&narrowed)
            .unwrap()
            .element_bounds(&descriptor, &f.registry),
        Ok((1, 3))
    );
    narrowed.products = Products::default();
    assert_eq!(
        actual
            .join(&narrowed)
            .unwrap()
            .element_bounds(&descriptor, &f.registry),
        Err(E::UnprovedAllocationSize)
    );
}
