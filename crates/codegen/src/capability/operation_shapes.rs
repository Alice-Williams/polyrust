//! Target-independent operand shapes needed for exact capability selection.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EqualityOperandShape {
    GeneralValue,
    PayloadFreeEnum,
    PayloadEnum,
}

pub(super) fn intrinsic_feature<T>(
    program: &CoreProgram,
    intrinsic: &CoreIntrinsicExpr<T>,
    operand_type: impl Fn(&T) -> CoreTypeId,
) -> (OperationFeature, FeatureShape) {
    match intrinsic {
        CoreIntrinsicExpr::Unary { operation, .. } => {
            (OperationFeature::Unary(*operation), FeatureShape::Unit)
        }
        CoreIntrinsicExpr::Binary {
            operation, left, ..
        } => {
            let shape = if matches!(
                operation,
                CoreBinaryIntrinsic::Equal | CoreBinaryIntrinsic::NotEqual
            ) {
                FeatureShape::Equality(equality_operand(program, operand_type(left)))
            } else {
                FeatureShape::Unit
            };
            (OperationFeature::Binary(*operation), shape)
        }
        CoreIntrinsicExpr::Ternary { operation, .. } => {
            (OperationFeature::Ternary(*operation), FeatureShape::Unit)
        }
        CoreIntrinsicExpr::Variadic {
            operation,
            arguments,
        } => (
            OperationFeature::Variadic(*operation),
            FeatureShape::Variadic {
                operand_count: usize_to_u32(arguments.len()),
            },
        ),
    }
}

fn equality_operand(program: &CoreProgram, operand: CoreTypeId) -> EqualityOperandShape {
    let Some(CoreType::Enum(id)) = program.types().get(operand) else {
        return EqualityOperandShape::GeneralValue;
    };
    let enumeration = program.enumeration(*id).expect("verified equality enum");
    if enumeration.variants.iter().all(|variant| {
        program
            .variant(*variant)
            .expect("verified equality variant")
            .fields
            .is_empty()
    }) {
        EqualityOperandShape::PayloadFreeEnum
    } else {
        EqualityOperandShape::PayloadEnum
    }
}
