//! Closed access traversal shared by lexical and initialization analyses.

use super::super::{
    CBinaryOperator, CCall, CCallableKind, CIndexBase, CInitializer, CInitializerKind, CPlace,
    CPlaceKind, CPointerTest, CValue, CValueKind,
};
use super::CContextError as E;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Access {
    Read,
    Write,
    Address,
}

pub(super) trait Visitor {
    fn evaluation(&self) -> Evaluation {
        Evaluation::AllSyntax
    }
    fn place(&mut self, place: &CPlace, access: Access) -> Result<(), E>;
    fn value(&mut self, _value: &CValue) -> Result<(), E> {
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Evaluation {
    AllSyntax,
    RuntimePaths,
}

pub(super) fn place(visitor: &mut impl Visitor, value: &CPlace, access: Access) -> Result<(), E> {
    visitor.place(value, access)?;
    match value.kind() {
        CPlaceKind::Local(_) | CPlaceKind::Parameter(_) | CPlaceKind::Global(_) => {}
        CPlaceKind::Member { base, .. } => place(visitor, base, Access::Address)?,
        CPlaceKind::Dereference(pointer) => expression(visitor, pointer)?,
        CPlaceKind::Index { base, index } => {
            match base {
                CIndexBase::Array(base) => place(visitor, base, Access::Address)?,
                CIndexBase::Pointer(pointer) => expression(visitor, pointer)?,
            }
            expression(visitor, index)?;
        }
    }
    Ok(())
}

pub(super) fn expression(visitor: &mut impl Visitor, value: &CValue) -> Result<(), E> {
    visitor.value(value)?;
    match value.kind() {
        CValueKind::Read(value) => place(visitor, value, Access::Read)?,
        CValueKind::AddressOf(value) => place(visitor, value, Access::Address)?,
        CValueKind::Call(value) => call(visitor, value)?,
        CValueKind::Unary { operand, .. } | CValueKind::Convert { operand, .. } => {
            expression(visitor, operand)?
        }
        CValueKind::Binary {
            operator,
            left,
            right,
        } => {
            expression(visitor, left)?;
            let skipped = matches!(
                (operator, super::constant_leaves::truth(left)),
                (CBinaryOperator::LogicalAnd, Some(false))
                    | (CBinaryOperator::LogicalOr, Some(true))
            );
            if visitor.evaluation() == Evaluation::AllSyntax || !skipped {
                expression(visitor, right)?;
            }
        }
        CValueKind::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            expression(visitor, condition)?;
            let selected = super::constant_leaves::truth(condition);
            if visitor.evaluation() == Evaluation::AllSyntax || selected != Some(false) {
                expression(visitor, then_value)?;
            }
            if visitor.evaluation() == Evaluation::AllSyntax || selected != Some(true) {
                expression(visitor, else_value)?;
            }
        }
        CValueKind::PointerTest(test) => match test {
            CPointerTest::IsNull(value) | CPointerTest::IsNonNull(value) => {
                expression(visitor, value)?
            }
            CPointerTest::SameSlot { left, right } => {
                expression(visitor, left)?;
                expression(visitor, right)?;
            }
        },
        CValueKind::KnownConstant(_)
        | CValueKind::Literal(_)
        | CValueKind::Enumerator(_)
        | CValueKind::FunctionAddress(_)
        | CValueKind::SizeOf(_)
        | CValueKind::AlignOf(_) => {}
    }
    Ok(())
}

pub(super) fn call(visitor: &mut impl Visitor, value: &CCall) -> Result<(), E> {
    if let CCallableKind::Indirect(pointer) = value.callable().kind() {
        expression(visitor, pointer)?;
    }
    for argument in value.arguments() {
        expression(visitor, argument)?;
    }
    Ok(())
}

pub(super) fn initializer(visitor: &mut impl Visitor, value: &CInitializer) -> Result<(), E> {
    match value.kind() {
        CInitializerKind::Expression(value) => expression(visitor, value)?,
        CInitializerKind::Zero(_) => {}
        CInitializerKind::Array { elements, .. } => {
            for value in elements {
                initializer(visitor, value)?;
            }
        }
        CInitializerKind::Struct { members, .. } => {
            for (_, value) in members {
                initializer(visitor, value)?;
            }
        }
        CInitializerKind::Union { value, .. } => initializer(visitor, value)?,
    }
    Ok(())
}
