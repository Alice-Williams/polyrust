//! Constant-expression categories only; 02C/02D also prove values and safety.

use super::{
    CConversion, CIndexBase, CInitializer, CInitializerKind, CLiteral, CObjectTypeKind, CPlace,
    CPlaceKind, CValue, CValueKind as V,
};

pub(super) fn is_integer_constant_expression(value: &CValue) -> bool {
    let integer = match value.ty().kind() {
        CObjectTypeKind::Scalar(value) => value.integer_promotion().is_some(),
        CObjectTypeKind::Enum(_) => true,
        _ => false,
    };
    integer
        && match value.kind() {
            V::Literal(
                CLiteral::Bool(_)
                | CLiteral::Signed(_)
                | CLiteral::Unsigned(_)
                | CLiteral::CharByte(_),
            )
            | V::Enumerator(_)
            | V::SizeOf(_)
            | V::AlignOf(_) => true,
            V::KnownConstant(value) => value.is_integer_constant_expression(),
            V::Unary { operand, .. }
            | V::Convert {
                conversion: CConversion::Numeric(_),
                operand,
            } => is_integer_constant_expression(operand),
            V::Binary { left, right, .. } => {
                is_integer_constant_expression(left) && is_integer_constant_expression(right)
            }
            V::Conditional {
                condition,
                then_value,
                else_value,
            } => {
                is_integer_constant_expression(condition)
                    && is_integer_constant_expression(then_value)
                    && is_integer_constant_expression(else_value)
            }
            _ => false,
        }
}

fn is_arithmetic_constant_expression(value: &CValue) -> bool {
    match value.kind() {
        V::Literal(
            CLiteral::Bool(_) | CLiteral::Signed(_) | CLiteral::Unsigned(_) | CLiteral::CharByte(_),
        )
        | V::Enumerator(_)
        | V::SizeOf(_)
        | V::AlignOf(_) => true,
        V::KnownConstant(value) => value.is_integer_constant_expression(),
        V::Unary { operand, .. } => is_arithmetic_constant_expression(operand),
        V::Binary { left, right, .. } => {
            is_arithmetic_constant_expression(left) && is_arithmetic_constant_expression(right)
        }
        V::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            is_arithmetic_constant_expression(condition)
                && is_arithmetic_constant_expression(then_value)
                && is_arithmetic_constant_expression(else_value)
        }
        V::Convert {
            conversion: CConversion::Numeric(_),
            operand,
        } => is_arithmetic_constant_expression(operand),
        V::Literal(CLiteral::NullPointer(_))
        | V::Read(_)
        | V::FunctionAddress(_)
        | V::Call(_)
        | V::PointerTest(_)
        | V::AddressOf(_)
        | V::Convert { .. } => false,
    }
}

fn is_address_constant(value: &CValue) -> bool {
    match value.kind() {
        V::Literal(CLiteral::NullPointer(_)) | V::FunctionAddress(_) => true,
        V::AddressOf(place) => is_static_place(place),
        V::Convert {
            conversion: CConversion::AddConst(_) | CConversion::ObjectToVoid(_),
            operand,
        } => is_address_constant(operand),
        _ => false,
    }
}

fn is_static_place(place: &CPlace) -> bool {
    match place.kind() {
        CPlaceKind::Global(_) => true,
        CPlaceKind::Member { base, .. } => is_static_place(base),
        CPlaceKind::Index {
            base: CIndexBase::Array(base),
            index,
        } => is_static_place(base) && is_integer_constant_expression(index),
        CPlaceKind::Local(_)
        | CPlaceKind::Parameter(_)
        | CPlaceKind::Dereference(_)
        | CPlaceKind::Index {
            base: CIndexBase::Pointer(_),
            ..
        } => false,
    }
}

pub(super) fn is_static_initializer(value: &CInitializer) -> bool {
    match value.kind() {
        CInitializerKind::Zero(_) => true,
        CInitializerKind::Expression(value) => {
            is_arithmetic_constant_expression(value) || is_address_constant(value)
        }
        CInitializerKind::Array { elements, .. } => elements.iter().all(is_static_initializer),
        CInitializerKind::Struct { members, .. } => members
            .iter()
            .all(|(_, value)| is_static_initializer(value)),
        CInitializerKind::Union { value, .. } => is_static_initializer(value),
    }
}
