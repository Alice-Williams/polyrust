//! Original numeric checks cannot be replaced by a later guard or equal snapshot.
use super::{product_fixture::*, *};
use crate::ast::{numeric_fixture::Fixture, *};

#[test]
fn runtime_counts_require_positive_original_bytes_and_precomputation_overflow_guards() {
    for late in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let count = count(&mut f, "count");
        let bytes = immutable(&mut f, "bytes", CScalarType::Size);
        let descriptor = buffer(
            &mut f,
            "buffer",
            &count,
            CObjectType::scalar(CScalarType::I64),
        );
        let positive = f.compare(CBinaryOperator::Greater, f.read(count.local()), f.size(0));
        let limit = f.compare(
            CBinaryOperator::LessEqual,
            f.read(count.local()),
            f.size(u64::MAX / 8),
        );
        let positive = assume(&mut f, positive);
        let limit = assume(&mut f, limit);
        let declaration = f.declare(&bytes, multiply(&f, f.read(count.local()), f.size(8)));
        let mut body = vec![f.declare(count.local(), f.input(0)), positive];
        if late {
            body.extend([declaration, limit]);
        } else {
            body.extend([limit, declaration]);
        }
        body.push(allocate(&f, f.read(&bytes)));
        let result = requests(&f, body);
        if late {
            assert_eq!(result, Err(E::UnprovedSizeArithmetic));
        } else {
            assert_eq!(
                result.unwrap()[0].element_bounds(&descriptor, &f.registry),
                Ok((1, u64::MAX / 8))
            );
        }
    }
}

#[test]
fn zero_and_wrapped_products_never_establish_live_elements() {
    for count_value in [0, u64::MAX / 8 + 1, u64::MAX] {
        let mut f = Fixture::new(&[]);
        let count = count(&mut f, "count");
        let result = requests(
            &f,
            vec![
                f.declare(count.local(), f.size(count_value)),
                allocate(&f, multiply(&f, f.read(count.local()), f.size(8))),
            ],
        );
        assert_eq!(
            result,
            Err(if count_value == 0 {
                E::UnprovedAllocationSize
            } else {
                E::UnprovedSizeArithmetic
            })
        );
    }
}

#[test]
fn mutable_source_changes_do_not_replace_the_captured_count() {
    let mut f = Fixture::new(&[]);
    let source = f.local(CScalarType::Size, "source");
    let count = count(&mut f, "count");
    let descriptor = buffer(
        &mut f,
        "buffer",
        &count,
        CObjectType::scalar(CScalarType::I64),
    );
    let actual = requests(
        &f,
        vec![
            f.declare(&source, f.size(2)),
            f.declare(count.local(), f.read(&source)),
            f.ast()
                .assign(f.values().local(source.clone()).unwrap(), f.size(100))
                .unwrap(),
            allocate(&f, multiply(&f, f.read(count.local()), f.size(8))),
            f.ast()
                .assign(f.values().local(source).unwrap(), f.size(0))
                .unwrap(),
        ],
    )
    .unwrap()
    .remove(0);
    assert_eq!(actual.bytes, (16, 16));
    assert_eq!(actual.element_bounds(&descriptor, &f.registry), Ok((2, 2)));
}
