//! Java AST: expression nodes.

use super::blocks::JavaBlock;
use super::declaration_model::JavaParameter;
use super::expression_model::{
    JavaBinaryOperator, JavaCallableRef, JavaLiteral, JavaPrecedence, JavaUnaryOperator,
    JavaValueRef,
};
use super::identifiers::JavaIdentifier;
use super::interface_witness::JavaInterfaceWitness;
use super::types::{JavaArrayOwnershipTransition, JavaType};
use portable_codegen::GeneratedTypeId;
use portable_core_ir::CoreFieldId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaConstructorRef {
    Known {
        constructor: crate::dialect::JavaKnownConstructor,
        owner: JavaType,
        parameters: Vec<JavaType>,
    },
    Generated {
        owner: GeneratedTypeId,
        parameters: Vec<JavaType>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaFieldRef {
    Known(crate::dialect::JavaKnownField),
    Structural {
        name: JavaIdentifier,
        ty: JavaType,
    },
    Generated {
        owner: GeneratedTypeId,
        field: CoreFieldId,
        name: JavaIdentifier,
        ty: JavaType,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaExpr {
    pub ty: JavaType,
    pub precedence: JavaPrecedence,
    pub kind: JavaExprKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JavaExprKind {
    Literal(JavaLiteral),
    Value(JavaValueRef),
    Unary {
        operator: JavaUnaryOperator,
        operand: Box<JavaExpr>,
    },
    Binary {
        operator: JavaBinaryOperator,
        left: Box<JavaExpr>,
        right: Box<JavaExpr>,
    },
    Conditional {
        condition: Box<JavaExpr>,
        when_true: Box<JavaExpr>,
        when_false: Box<JavaExpr>,
    },
    Call {
        callable: JavaCallableRef,
        receiver: Option<Box<JavaExpr>>,
        arguments: Vec<JavaExpr>,
    },
    New {
        constructor: JavaConstructorRef,
        arguments: Vec<JavaExpr>,
    },
    NewArray {
        component: JavaType,
        length: Box<JavaExpr>,
    },
    ArrayIndex {
        array: Box<JavaExpr>,
        index: Box<JavaExpr>,
    },
    Field {
        receiver: Box<JavaExpr>,
        field: JavaFieldRef,
    },
    Cast {
        target: JavaType,
        value: Box<JavaExpr>,
    },
    InterfaceCoercion {
        implementation: JavaInterfaceWitness,
        target: JavaType,
        value: Box<JavaExpr>,
    },
    ArrayOwnershipTransition {
        transition: JavaArrayOwnershipTransition,
        value: Box<JavaExpr>,
    },
    InstanceOf {
        value: Box<JavaExpr>,
        target: JavaType,
        binding: Option<JavaIdentifier>,
    },
    Lambda {
        parameters: Vec<JavaParameter>,
        body: JavaBlock,
    },
}
