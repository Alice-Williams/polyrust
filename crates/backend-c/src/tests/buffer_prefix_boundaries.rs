//! Empty paths and maximum bounds are proved without executing enormous loops.
use super::{buffer_fixture::Buffer, buffer_prefix_fixture::Counted, numeric_fixture::Fixture, *};

#[test]
fn zero_length_has_an_explicit_no_allocation_path() {
    let mut f = Fixture::new(&[CScalarType::Size]);
    let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Size));
    let construction = Counted::new(&mut f, "construct");
    let empty = f.compare(CBinaryOperator::Equal, f.input(0), f.size(0));
    let mut body = vec![f.branch(empty, vec![f.ast().return_statement(None).unwrap()], vec![])];
    let too_large = f.compare(CBinaryOperator::Greater, f.input(0), f.size(u64::MAX / 8));
    body.push(f.branch(
        too_large,
        vec![f.ast().return_statement(None).unwrap()],
        vec![],
    ));
    let count = f.input(0);
    body.extend(buffer.prefix(&mut f, count));
    body.push(construction.declaration(&f));
    let place = buffer.place(&f, f.read(&construction.counter));
    body.push(construction.finish(
        &f,
        buffer.count.local(),
        vec![
            f.ast().assign(place, f.size(7)).unwrap(),
            construction.step(&f),
        ],
    ));
    body.push(f.discard(buffer.read(&f, 0)));
    body.push(buffer.release(&f));
    assert_eq!(f.registry.check_storage_paths(&[f.source(body)]), Ok(()));
}

#[test]
fn byte_prefixes_accept_one_and_maximum_nonwrapping_counts() {
    for count in [1, u64::MAX] {
        let mut f = Fixture::new(&[]);
        let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::U8));
        let construction = Counted::new(&mut f, "construct");
        let size = f.size(count);
        let mut body = buffer.prefix(&mut f, size);
        body.push(construction.declaration(&f));
        let value = f
            .values()
            .numeric_conversion(CScalarType::U8, f.size(7))
            .unwrap();
        body.push(construction.finish(
            &f,
            buffer.count.local(),
            vec![
            f.ast().assign(buffer.place(&f, f.read(&construction.counter)), value).unwrap(),
            construction.step(&f),
        ],
        ));
        body.push(f.discard(buffer.read(&f, count - 1)));
        body.push(buffer.release(&f));
        assert_eq!(
            f.registry.check_storage_paths(&[f.source(body)]),
            Ok(()),
            "count={count}"
        );
    }
}
