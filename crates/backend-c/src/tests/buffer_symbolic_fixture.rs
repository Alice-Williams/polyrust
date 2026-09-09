//! Runtime counts and index parameters keep symbolic tests non-constant.
use super::{buffer_fixture::Buffer, numeric_fixture::Fixture, *};

pub(super) fn setup() -> (Fixture, Buffer, Vec<CStatement>) {
    let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size, CScalarType::Size]);
    let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Size));
    let lower = f.compare(CBinaryOperator::GreaterEqual, f.input(0), f.size(2));
    let upper = f.compare(CBinaryOperator::LessEqual, f.input(0), f.size(u64::MAX / 8));
    let condition = f.boolean(f.binary(CBinaryOperator::LogicalAnd, lower, upper));
    let guard = f.branch(
        condition,
        vec![],
        vec![f.ast().return_statement(None).unwrap()],
    );
    let mut body = vec![guard];
    let count = f.input(0);
    body.extend(buffer.prefix(&mut f, count));
    (f, buffer, body)
}
pub(super) fn selected(f: &Fixture, buffer: &Buffer, index: CValue) -> CValue {
    f.values().read(buffer.place(f, index)).unwrap()
}
pub(super) fn write(f: &Fixture, buffer: &Buffer, index: CValue, value: CValue) -> CStatement {
    f.ast().assign(buffer.place(f, index), value).unwrap()
}
pub(super) fn within(
    f: &mut Fixture,
    buffer: &Buffer,
    index: CValue,
    actions: Vec<CStatement>,
) -> CStatement {
    let condition = f.compare(CBinaryOperator::Less, index, f.read(buffer.count.local()));
    f.branch(condition, actions, vec![])
}
pub(super) fn positive_small(f: &Fixture, value: CValue) -> CValue {
    let lower = f.compare(CBinaryOperator::Greater, value.clone(), f.size(0));
    let upper = f.compare(CBinaryOperator::LessEqual, value, f.size(16));
    f.boolean(f.binary(CBinaryOperator::LogicalAnd, lower, upper))
}
