//! Recheck declared types, complete inventories and every initializer child.

use super::super::{CInitializer, CInitializerKind};
use super::{CContextError as E, Recheck};

impl Recheck<'_> {
    pub(super) fn initializer(&self, value: &CInitializer) -> Result<CInitializer, E> {
        let ast = &self.expressions;
        let rebuilt = match value.kind() {
            CInitializerKind::Expression(value) => {
                ast.expression_initializer(self.value(value)?)?
            }
            CInitializerKind::Zero(ty) => ast.zero_initializer(ty.clone())?,
            CInitializerKind::Array {
                declared_type,
                elements,
            } => ast.array_initializer(
                declared_type.clone(),
                elements
                    .iter()
                    .map(|value| self.initializer(value))
                    .collect::<Result<Vec<_>, _>>()?,
            )?,
            CInitializerKind::Struct { owner, members } => ast.struct_initializer(
                owner.clone(),
                members
                    .iter()
                    .map(|(member, value)| Ok((member.clone(), self.initializer(value)?)))
                    .collect::<Result<Vec<_>, E>>()?,
            )?,
            CInitializerKind::Union {
                owner,
                member,
                value,
            } => ast.union_initializer(owner.clone(), member.clone(), self.initializer(value)?)?,
        };
        if value.ty() != rebuilt.ty() || value.brand != rebuilt.brand {
            return Err(E::StoredStructureMismatch);
        }
        Ok(rebuilt)
    }
}
