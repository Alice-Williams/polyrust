//! Java mapping for `BooleanLogic`.

mod mapping_plan;

use portable_build::BooleanLogic;

use super::support::{JavaMappingOutput, java_operation_mapping, sealed};
use crate::{
    ast::{
        JavaBinaryOperator, JavaBlock, JavaExpr, JavaIdentifier, JavaLocalFinality, JavaStmt,
        JavaType, JavaUnaryOperator,
    },
    lower::{binary, bool_literal, unary},
};

#[doc(hidden)]
#[derive(Clone)]
pub struct JavaBooleanLogicPlan {
    pub(crate) statements: Vec<JavaStmt>,
    pub(crate) value: JavaExpr,
}

impl sealed::JavaMappingOutput for JavaBooleanLogicPlan {}
impl JavaMappingOutput for JavaBooleanLogicPlan {}

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaBooleanLogicInput {
    Not {
        operand: JavaBooleanLogicPlan,
        result: JavaType,
    },
    And {
        left: JavaBooleanLogicPlan,
        right: JavaBooleanLogicPlan,
        result_name: JavaIdentifier,
        result: JavaType,
    },
    Or {
        left: JavaBooleanLogicPlan,
        right: JavaBooleanLogicPlan,
        result_name: JavaIdentifier,
        result: JavaType,
    },
}

fn lower_boolean_logic(
    input: JavaBooleanLogicInput,
) -> Result<JavaBooleanLogicPlan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaBooleanLogicInput::Not {
            mut operand,
            result,
        } => {
            operand.value = unary(JavaUnaryOperator::Not, operand.value, result);
            operand
        }
        JavaBooleanLogicInput::And {
            left,
            right,
            result_name,
            result,
        } => short_circuit(left, right, result_name, result, false),
        JavaBooleanLogicInput::Or {
            left,
            right,
            result_name,
            result,
        } => short_circuit(left, right, result_name, result, true),
    })
}

fn short_circuit(
    left: JavaBooleanLogicPlan,
    right: JavaBooleanLogicPlan,
    result_name: JavaIdentifier,
    result_type: JavaType,
    when_left: bool,
) -> JavaBooleanLogicPlan {
    if right.statements.is_empty() {
        return JavaBooleanLogicPlan {
            statements: left.statements,
            value: binary(
                if when_left {
                    JavaBinaryOperator::LogicalOr
                } else {
                    JavaBinaryOperator::LogicalAnd
                },
                left.value,
                right.value,
                result_type,
            ),
        };
    }
    let result = JavaExpr::local(result_type.clone(), result_name.clone());
    let right_block = JavaBlock::new(
        right
            .statements
            .into_iter()
            .chain([JavaStmt::Assign {
                target: result.clone(),
                value: right.value,
            }])
            .collect(),
    );
    let condition = if when_left {
        left.value
    } else {
        unary(JavaUnaryOperator::Not, left.value, result_type.clone())
    };
    let mut statements = left.statements;
    statements.push(JavaStmt::Local {
        finality: JavaLocalFinality::Mutable,
        ty: result_type,
        name: result_name,
        value: None,
    });
    statements.push(JavaStmt::If {
        condition,
        then_block: JavaBlock::new(vec![JavaStmt::Assign {
            target: result.clone(),
            value: bool_literal(when_left),
        }]),
        else_block: Some(right_block),
    });
    JavaBooleanLogicPlan {
        statements,
        value: result,
    }
}

java_operation_mapping!(
    JavaBooleanLogic,
    BooleanLogic,
    JavaBooleanLogicInput,
    JavaBooleanLogicPlan,
    lower_boolean_logic
);
