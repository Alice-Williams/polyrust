//! Closed access traversal shared by lexical and initialization analyses.

use super::super::{
    CBinaryOperator, CCall, CCallableKind, CIndexBase, CInitializer, CInitializerKind, CPlace,
    CPlaceKind, CPointerTest, CValue, CValueKind,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Access {
    Read,
    Write,
    Address,
}

pub(crate) trait Visitor {
    type Error;
    fn evaluation(&self) -> Evaluation {
        Evaluation::AllSyntax
    }
    fn place(&mut self, place: &CPlace, access: Access) -> Result<(), Self::Error>;
    fn value(&mut self, _value: &CValue) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Evaluation {
    AllSyntax,
    RuntimePaths,
}

pub(crate) fn place<V: Visitor>(
    visitor: &mut V,
    value: &CPlace,
    access: Access,
) -> Result<(), V::Error> {
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

pub(crate) fn expression<V: Visitor>(visitor: &mut V, value: &CValue) -> Result<(), V::Error> {
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

pub(crate) fn call<V: Visitor>(visitor: &mut V, value: &CCall) -> Result<(), V::Error> {
    match value.callable().kind() {
        CCallableKind::Indirect { pointer, .. } => expression(visitor, pointer)?,
        CCallableKind::Direct(_) | CCallableKind::Known(_) => {}
    }
    for argument in value.arguments() {
        expression(visitor, argument)?;
    }
    Ok(())
}

pub(crate) fn initializer<V: Visitor>(
    visitor: &mut V,
    value: &CInitializer,
) -> Result<(), V::Error> {
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
