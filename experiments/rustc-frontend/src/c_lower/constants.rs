//! Owned scalar constant inventory and ordinary header/source declarations.
use super::{Result, c};
use crate::source_capabilities::ScalarConstantValue;
use portable_backend_c::ast::*;
use rustc_hir::def_id::DefId;
use std::collections::HashMap;

pub(crate) type OwnedConstants = HashMap<DefId, (CObjectRef, ScalarConstantValue)>;

pub(super) fn value(input: ScalarConstantValue) -> CScalarConstantValue {
    match input {
        ScalarConstantValue::Bool(value) => CScalarConstantValue::Bool(value),
        ScalarConstantValue::I32(value) => CScalarConstantValue::I32(value),
        ScalarConstantValue::I64(value) => CScalarConstantValue::I64(value),
        ScalarConstantValue::Char(value) => CScalarConstantValue::U32(u32::from(value)),
        ScalarConstantValue::F64(value) => CScalarConstantValue::F64(value),
        ScalarConstantValue::Infinity(sign) => CScalarConstantValue::Infinity(sign),
    }
}

pub(super) fn expression(registry: &CRegistry, input: ScalarConstantValue) -> Result<CValue> {
    let expressions = CExpressions::new(registry);
    match input {
        ScalarConstantValue::Bool(value) => c(expressions.literal(CLiteral::Bool(value))),
        ScalarConstantValue::I32(value) => {
            c(expressions.literal(CLiteral::Signed(CSignedLiteral::I32(value))))
        }
        ScalarConstantValue::I64(value) => {
            c(expressions.literal(CLiteral::Signed(CSignedLiteral::I64(value))))
        }
        ScalarConstantValue::F64(value) => c(expressions.literal(CLiteral::F64(value))),
        ScalarConstantValue::Char(value) => {
            c(expressions.literal(CLiteral::Unsigned(CUnsignedLiteral::U32(u32::from(value)))))
        }
        ScalarConstantValue::Infinity(sign) => {
            let value = expressions.known_constant(CKnownConstant::DoubleInfinity);
            match sign {
                portable_binary64::Binary64Sign::Positive => Ok(value),
                portable_binary64::Binary64Sign::Negative => {
                    c(expressions.unary(CUnaryOperator::Negate, value))
                }
            }
        }
    }
}

pub(super) fn files(state: &super::package::State) -> Result<(Vec<CFileItem>, Vec<CFileItem>)> {
    let mut header_items = Vec::new();
    let mut definitions = Vec::new();
    let mut constants: Vec<_> = state.constants.values().collect();
    constants.sort_by_key(|(object, _)| object.key().clone());
    for (object, value) in constants {
        header_items.push(CFileItem::Declaration(c(c(CDeclarations::new(
            &state.registry,
            object.file().clone(),
        ))?
        .object_declaration(object.clone()))?));
        let value = expression(&state.registry, *value)?;
        definitions.push(CFileItem::Definition(c(c(CDeclarations::new(
            &state.registry,
            state.file.clone(),
        ))?
        .object_definition(
            object.clone(),
            CLinkage::External,
            c(CExpressions::new(&state.registry).expression_initializer(value))?,
        ))?));
    }
    Ok((header_items, definitions))
}

/// Imported objects retain their exact independent owner witness across bodies.
pub(super) type ImportedConstants =
    HashMap<DefId, (CObjectRef, portable_backend_c::dialect::CDependencyConstant)>;
