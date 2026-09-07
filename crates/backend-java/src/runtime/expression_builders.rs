//! Typed runtime construction: expression builders.

use super::*;

pub(super) fn static_method(
    type_parameters: Vec<JavaIdentifier>,
    return_type: JavaType,
    name: &str,
    parameters: Vec<JavaParameter>,
    statements: Vec<JavaStmt>,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Static],
        type_parameters,
        return_type,
        name: identifier(name),
        parameters,
        body: Some(JavaBlock::new(statements)),
    })
}

pub(super) fn package_static_method(
    type_parameters: Vec<JavaIdentifier>,
    return_type: JavaType,
    name: &str,
    parameters: Vec<JavaParameter>,
    statements: Vec<JavaStmt>,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Static],
        type_parameters,
        return_type,
        name: identifier(name),
        parameters,
        body: Some(JavaBlock::new(statements)),
    })
}

pub(super) fn private_final_field(ty: JavaType, name: &str) -> JavaMember {
    JavaMember::Field(JavaField {
        declared: None,
        modifiers: vec![JavaModifier::Private, JavaModifier::Final],
        ty,
        name: identifier(name),
        initializer: None,
    })
}

pub(super) fn field_accessor(
    owner: JavaType,
    name: &str,
    ty: JavaType,
    member: JavaRuntimeMember,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public],
        type_parameters: vec![],
        return_type: ty.clone(),
        name: identifier(member.name()),
        parameters: vec![],
        body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
            structural_field(this_value(owner), name, ty),
        ))])),
    })
}
pub(super) fn local(ty: JavaType, name: &str) -> JavaExpr {
    JavaExpr::local(ty, identifier(name))
}
pub(super) fn unary(operator: JavaUnaryOperator, operand: JavaExpr, ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator,
            operand: Box::new(operand),
        },
    }
}
pub(super) fn binary(
    operator: JavaBinaryOperator,
    left: JavaExpr,
    right: JavaExpr,
    ty: JavaType,
) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: match operator {
            JavaBinaryOperator::LogicalOr => JavaPrecedence::LogicalOr,
            JavaBinaryOperator::LogicalAnd => JavaPrecedence::LogicalAnd,
            JavaBinaryOperator::BitOr => JavaPrecedence::BitOr,
            JavaBinaryOperator::BitXor => JavaPrecedence::BitXor,
            JavaBinaryOperator::BitAnd => JavaPrecedence::BitAnd,
            JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual => JavaPrecedence::Equality,
            JavaBinaryOperator::Less
            | JavaBinaryOperator::LessEqual
            | JavaBinaryOperator::Greater
            | JavaBinaryOperator::GreaterEqual => JavaPrecedence::Relational,
            JavaBinaryOperator::ShiftLeft | JavaBinaryOperator::ShiftRight => JavaPrecedence::Shift,
            JavaBinaryOperator::Add | JavaBinaryOperator::Subtract => JavaPrecedence::Additive,
            JavaBinaryOperator::Multiply
            | JavaBinaryOperator::Divide
            | JavaBinaryOperator::Remainder => JavaPrecedence::Multiplicative,
        },
        kind: JavaExprKind::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        },
    }
}
pub(super) fn conditional(
    condition: JavaExpr,
    when_true: JavaExpr,
    when_false: JavaExpr,
    ty: JavaType,
) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(condition),
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
        },
    }
}
pub(super) fn cast(target: JavaType, value: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: target.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target,
            value: Box::new(value),
        },
    }
}
pub(super) fn fresh_copy_to_boundary(value: JavaExpr, target: JavaType) -> JavaExpr {
    JavaExpr {
        ty: target,
        precedence: value.precedence,
        kind: JavaExprKind::ArrayOwnershipTransition {
            transition: JavaArrayOwnershipTransition::FreshCopyToBoundary,
            value: Box::new(value),
        },
    }
}
pub(super) fn instance_of(
    value: JavaExpr,
    target: JavaType,
    binding: Option<JavaIdentifier>,
) -> JavaExpr {
    JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Boolean),
        precedence: JavaPrecedence::Relational,
        kind: JavaExprKind::InstanceOf {
            value: Box::new(value),
            target,
            binding,
        },
    }
}
pub(super) fn new_array(component: JavaType, length: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: JavaType::Array {
            component: Box::new(component.clone()),
            ownership: JavaArrayOwnership::InternalMutable,
        },
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::NewArray {
            component,
            length: Box::new(length),
        },
    }
}
pub(super) fn array_index(array: JavaExpr, index: JavaExpr, component: JavaType) -> JavaExpr {
    JavaExpr {
        ty: component,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::ArrayIndex {
            array: Box::new(array),
            index: Box::new(index),
        },
    }
}
pub(super) fn array_length(array: JavaExpr) -> JavaExpr {
    structural_field(array, "length", JavaType::primitive(JavaPrimitive::Int))
}
pub(super) fn structural_field(receiver: JavaExpr, name: &str, ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty: ty.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Field {
            receiver: Box::new(receiver),
            field: JavaFieldRef::Structural {
                name: identifier(name),
                ty,
            },
        },
    }
}
pub(super) fn this_value(ty: JavaType) -> JavaExpr {
    JavaExpr {
        ty,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::This),
    }
}
pub(super) fn assign_component(
    owner: JavaType,
    name: &str,
    ty: JavaType,
    value: JavaExpr,
) -> JavaStmt {
    JavaStmt::Assign {
        target: structural_field(this_value(owner), name, ty),
        value,
    }
}
pub(super) fn illegal_argument(message: &str) -> JavaStmt {
    JavaStmt::Throw(new_known(
        JavaKnownConstructor::IllegalArgumentExceptionString,
        JavaType::known(JavaKnownType::IllegalArgumentException),
        vec![string_literal(message)],
    ))
}
pub(super) fn illegal_state(message: &str) -> JavaStmt {
    JavaStmt::Throw(new_known(
        JavaKnownConstructor::IllegalStateExceptionString,
        JavaType::known(JavaKnownType::IllegalStateException),
        vec![string_literal(message)],
    ))
}
pub(super) fn guarded_accessor(
    owner: JavaType,
    name: &str,
    ty: JavaType,
    invalid: JavaExpr,
    message: &str,
) -> JavaMember {
    JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public],
        type_parameters: vec![],
        return_type: ty.clone(),
        name: identifier(name),
        parameters: vec![],
        body: Some(JavaBlock::new(vec![
            JavaStmt::If {
                condition: invalid,
                then_block: JavaBlock::new(vec![illegal_state(message)]),
                else_block: None,
            },
            JavaStmt::Return(Some(structural_field(this_value(owner), name, ty))),
        ])),
    })
}
pub(super) fn bytes_values(receiver: JavaExpr, array_type: JavaType) -> JavaExpr {
    member_call(receiver, JavaRuntimeMember::BytesValues, vec![], array_type)
}
pub(super) fn bool_literal(value: bool) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Boolean),
        JavaLiteral::Boolean(value),
    )
}
pub(super) fn int_literal(value: i32) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Int),
        JavaLiteral::I32(value),
    )
}
pub(super) fn long_literal(value: i64) -> JavaExpr {
    JavaExpr::literal(
        JavaType::primitive(JavaPrimitive::Long),
        JavaLiteral::I64(value),
    )
}
pub(super) fn double_literal(bits: u64) -> JavaExpr {
    known_call(
        JavaKnownCallable::DoubleFromLongBits,
        vec![long_literal(bits as i64)],
    )
}
pub(super) fn string_literal(value: &str) -> JavaExpr {
    JavaExpr::literal(
        JavaType::known(JavaKnownType::String),
        JavaLiteral::String(value.to_owned()),
    )
}
pub(super) fn null_literal(ty: JavaType) -> JavaExpr {
    JavaExpr::literal(
        ty,
        JavaLiteral::InternalNull(JavaNullPurpose::AbsentTaggedPayload),
    )
}
