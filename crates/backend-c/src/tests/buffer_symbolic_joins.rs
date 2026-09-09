//! Must facts and nested bounds remain independent of a symbolic outer index.
use super::{
    buffer_fixture::Buffer, buffer_symbolic_fixture::*, numeric_fixture::Fixture,
    storage_fixture::*, *,
};

#[test]
fn symbolic_initialization_survives_only_when_both_branches_write() {
    for both in [false, true] {
        let (mut f, buffer, mut body) = setup();
        let condition = f.compare(
            CBinaryOperator::Less,
            f.input(1),
            f.read(buffer.count.local()),
        );
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        let yes = vec![write(&f, &buffer, f.input(1), f.size(7))];
        let no = if both {
            vec![write(&f, &buffer, f.input(1), f.size(9))]
        } else {
            vec![]
        };
        let condition = f.compare(CBinaryOperator::Greater, f.input(2), f.size(0));
        body.push(f.branch(condition, yes, no));
        body.push(f.discard(selected(&f, &buffer, f.input(1))));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if both {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}

#[test]
fn a_symbolic_outer_element_does_not_change_nested_array_initialization_or_bounds() {
    for inner in 0..3 {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let array = CObjectType::array(
            CObjectType::scalar(CScalarType::Int),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let buffer = Buffer::new(&mut f, array);
        let count = f.size(2);
        let mut body = buffer.prefix(&mut f, count);
        let condition = f.compare(
            CBinaryOperator::Less,
            f.input(0),
            f.read(buffer.count.local()),
        );
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        let element = |index| {
            f.values()
                .index(
                    CIndexBase::Array(Box::new(buffer.place(&f, f.input(0)))),
                    f.size(index),
                )
                .unwrap()
        };
        body.push(f.ast().assign(element(0), f.int(7)).unwrap());
        body.push(f.discard(f.values().read(element(inner)).unwrap()));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            match inner {
                0 => Ok(()),
                1 => Err(CSafetyError::UninitializedStorage),
                _ => Err(CSafetyError::IndexOutOfBounds),
            },
        );
    }
}
