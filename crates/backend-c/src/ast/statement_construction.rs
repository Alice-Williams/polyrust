//! Ordinary statements. Structural CFG and ownership proofs remain independent.

use super::{
    CBlock, CConstness, CEffect, CExpressions, CFunctionRef, CInitializer, CLocalDeclaration,
    CLocalRef, CObjectTypeKind, CPlace, CRegistry, CReturnType, CScopeRef, CStatement,
    CStatementError as E, CStatementKind, CValue,
};

pub struct CStatements<'a> {
    pub(super) expressions: CExpressions<'a>,
    pub(super) function: CFunctionRef,
}

impl<'a> CStatements<'a> {
    pub fn new(registry: &'a CRegistry, function: CFunctionRef) -> Result<Self, E> {
        registry.check_function(&function)?;
        Ok(Self {
            expressions: CExpressions::new(registry),
            function,
        })
    }

    pub(super) fn statement(&self, kind: CStatementKind) -> CStatement {
        CStatement {
            function: self.function.clone(),
            kind,
        }
    }

    pub(super) fn check_scope(&self, scope: &CScopeRef) -> Result<(), E> {
        self.expressions.registry.check_lexical_scope(scope)?;
        if scope.function() != &self.function {
            return Err(E::WrongScope);
        }
        Ok(())
    }

    pub(super) fn check_block(&self, block: &CBlock) -> Result<(), E> {
        self.check_scope(block.scope())
    }

    pub fn empty(&self) -> CStatement {
        self.statement(CStatementKind::Empty)
    }

    pub fn block(&self, scope: CScopeRef, statements: Vec<CStatement>) -> Result<CBlock, E> {
        self.check_scope(&scope)?;
        for statement in &statements {
            if statement.function() != &self.function {
                return Err(E::WrongScope);
            }
            self.check_placement(&scope, statement)?;
        }
        Ok(CBlock { scope, statements })
    }

    pub fn nested_block(&self, block: CBlock) -> Result<CStatement, E> {
        self.check_block(&block)?;
        Ok(self.statement(CStatementKind::Block(block)))
    }

    pub fn declare(
        &self,
        local: CLocalRef,
        initializer: Option<CInitializer>,
    ) -> Result<CStatement, E> {
        self.expressions
            .registry
            .check_local(&self.function, &local)?;
        if let Some(value) = &initializer {
            self.expressions.initializer_fits(local.ty(), value)?;
        } else if self.expressions.registry.has_const_subobject(local.ty())? {
            return Err(E::ConstRequiresInitializer);
        }
        Ok(self.statement(CStatementKind::Declare(CLocalDeclaration {
            local,
            initializer,
        })))
    }

    pub fn assign(&self, place: CPlace, value: CValue) -> Result<CStatement, E> {
        self.expressions.check_place(&place)?;
        self.expressions.check_value(&value)?;
        if matches!(place.ty().kind(), CObjectTypeKind::Array { .. })
            || self.expressions.registry.has_const_subobject(place.ty())?
        {
            return Err(E::NotModifiable);
        }
        if !self
            .expressions
            .registry
            .types_match(place.ty(), value.ty())?
        {
            return Err(E::TypeMismatch);
        }
        Ok(self.statement(CStatementKind::Assign { place, value }))
    }

    pub fn evaluate(&self, effect: CEffect) -> Result<CStatement, E> {
        self.expressions.check_callable(effect.call().callable())?;
        Ok(self.statement(CStatementKind::Evaluate(effect)))
    }

    pub fn discard(&self, value: CValue) -> Result<CStatement, E> {
        self.expressions.check_value(&value)?;
        Ok(self.statement(CStatementKind::Discard(value)))
    }

    pub fn if_statement(
        &self,
        condition: CValue,
        then_block: CBlock,
        else_block: CBlock,
    ) -> Result<CStatement, E> {
        self.expressions.check_bool(&condition)?;
        self.check_block(&then_block)?;
        self.check_block(&else_block)?;
        Ok(self.statement(CStatementKind::If {
            condition,
            then_block,
            else_block,
        }))
    }

    pub fn return_statement(&self, value: Option<CValue>) -> Result<CStatement, E> {
        match (self.function.signature().return_type(), &value) {
            (CReturnType::Void, None) => {}
            (CReturnType::Value(expected), Some(value)) => {
                self.expressions.check_value(value)?;
                if !self
                    .expressions
                    .registry
                    .types_match(expected.ty(), value.ty())?
                {
                    return Err(E::TypeMismatch);
                }
            }
            _ => return Err(E::ReturnCategory),
        }
        Ok(self.statement(CStatementKind::Return(value)))
    }
}

impl CRegistry {
    pub(super) fn has_const_subobject(&self, ty: &super::CObjectType) -> Result<bool, E> {
        self.check_type(ty)?;
        let mut pending = vec![ty.canonical()];
        let mut visited = std::collections::BTreeSet::new();
        while let Some(ty) = pending.pop() {
            if ty.constness() == CConstness::Const {
                return Ok(true);
            }
            let owner = match ty.kind() {
                CObjectTypeKind::Array { element, .. } => {
                    pending.push((**element).clone());
                    continue;
                }
                CObjectTypeKind::Struct(value) => super::CAggregateRef::Struct(value.clone()),
                CObjectTypeKind::Union(value) => super::CAggregateRef::Union(value.clone()),
                _ => continue,
            };
            if visited.insert(owner.clone()) {
                pending.extend(
                    self.members(&owner)?
                        .ok_or(E::IncompleteAggregate)?
                        .iter()
                        .map(|member| member.ty().canonical()),
                );
            }
        }
        Ok(false)
    }
}
