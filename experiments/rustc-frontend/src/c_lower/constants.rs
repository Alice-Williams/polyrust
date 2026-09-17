//! Owned scalar constant inventory and ordinary header/source declarations.
use super::{Result, c};
use crate::source_capabilities::ScalarConstantValue;
use portable_backend_c::ast::*;
use rustc_hir::def_id::DefId;
use std::collections::HashMap;

pub(crate) type OwnedConstants = HashMap<DefId, (CObjectRef, ScalarConstantValue)>;

pub(super) fn literal(value: ScalarConstantValue) -> CLiteral {
    match value {
        ScalarConstantValue::Bool(value) => CLiteral::Bool(value),
        ScalarConstantValue::I32(value) => CLiteral::Signed(CSignedLiteral::I32(value)),
        ScalarConstantValue::I64(value) => CLiteral::Signed(CSignedLiteral::I64(value)),
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
        let value = c(CExpressions::new(&state.registry).literal(literal(*value)))?;
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
