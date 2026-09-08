//! Explicit control nodes: no hidden counted-loop update or label expansion.

use super::{
    CBlock, CBreakTarget, CCaseConstant, CCleanupExitRef, CConstness, CCountedProgress,
    CCountedStep, CLocalRef, CLoopRef, CObjectType, CScalarType, CStatement, CStatementError as E,
    CStatementKind, CStatements, CSwitchArm, CSwitchRef, CValue,
};

impl CStatements<'_> {
    pub fn counted_loop(
        &self,
        identity: CLoopRef,
        counter: CLocalRef,
        bound: CLocalRef,
        condition: CValue,
        body: CBlock,
    ) -> Result<CStatement, E> {
        self.check_scope(identity.scope())?;
        self.expressions
            .registry
            .check_loop(identity.scope(), &identity)?;
        self.expressions
            .registry
            .check_local(&self.function, &counter)?;
        self.expressions
            .registry
            .check_local(&self.function, &bound)?;
        let size = CObjectType::scalar(CScalarType::Size);
        let const_size = size
            .clone()
            .with_constness(CConstness::Const)
            .expect("scalar qualifier");
        if counter.scope() != identity.scope()
            || bound.scope() != identity.scope()
            || counter.ty().canonical() != size
            || bound.ty().canonical() != const_size
        {
            return Err(E::InvalidCountedProgress);
        }
        self.expressions.check_bool(&condition)?;
        self.check_block(&body)?;
        if body.scope().parent() != Some(identity.scope()) {
            return Err(E::WrongScope);
        }
        let progress = CCountedProgress {
            counter,
            bound,
            step: CCountedStep::One,
        };
        Ok(self.statement(CStatementKind::BoundedLoop {
            identity,
            progress: Box::new(progress),
            condition,
            body,
        }))
    }

    pub fn switch_arm(&self, cases: Vec<CCaseConstant>, body: CBlock) -> Result<CSwitchArm, E> {
        self.check_block(&body)?;
        if cases.is_empty() {
            return Err(E::EmptyCaseList);
        }
        for value in &cases {
            if let CCaseConstant::Enumerator(value) = value {
                self.expressions.registry.check_enumerator(value)?;
            }
        }
        Ok(CSwitchArm { cases, body })
    }

    pub fn switch_statement(
        &self,
        identity: CSwitchRef,
        value: CValue,
        arms: Vec<CSwitchArm>,
        default: CBlock,
    ) -> Result<CStatement, E> {
        self.check_scope(identity.scope())?;
        self.expressions
            .registry
            .check_switch(identity.scope(), &identity)?;
        if self
            .expressions
            .arithmetic_type(&value)?
            .integer_promotion()
            .is_none()
        {
            return Err(E::ExpectedIntegerSwitch);
        }
        self.check_block(&default)?;
        for arm in &arms {
            self.check_block(arm.body())?;
        }
        Ok(self.statement(CStatementKind::Switch {
            identity,
            value,
            arms,
            default,
        }))
    }

    pub fn break_statement(&self, target: CBreakTarget) -> Result<CStatement, E> {
        match &target {
            CBreakTarget::Loop(value) => {
                self.check_scope(value.scope())?;
                self.expressions.registry.check_loop(value.scope(), value)?;
            }
            CBreakTarget::Switch(value) => {
                self.check_scope(value.scope())?;
                self.expressions
                    .registry
                    .check_switch(value.scope(), value)?;
            }
        }
        Ok(self.statement(CStatementKind::Break(target)))
    }

    pub fn continue_statement(&self, identity: CLoopRef) -> Result<CStatement, E> {
        self.check_scope(identity.scope())?;
        self.expressions
            .registry
            .check_loop(identity.scope(), &identity)?;
        Ok(self.statement(CStatementKind::Continue(identity)))
    }

    pub fn cleanup_jump(&self, identity: CCleanupExitRef) -> Result<CStatement, E> {
        self.check_scope(identity.scope())?;
        self.expressions
            .registry
            .check_cleanup_exit(identity.scope(), &identity)?;
        Ok(self.statement(CStatementKind::CleanupJump(identity)))
    }

    pub fn label(&self, identity: CCleanupExitRef, statement: CStatement) -> Result<CStatement, E> {
        self.check_scope(identity.scope())?;
        self.expressions
            .registry
            .check_cleanup_exit(identity.scope(), &identity)?;
        if statement.function() != &self.function {
            return Err(E::WrongScope);
        }
        if matches!(statement.kind(), CStatementKind::Declare(_)) {
            return Err(E::LabelBeforeDeclaration);
        }
        Ok(self.statement(CStatementKind::Label {
            identity,
            statement: Box::new(statement),
        }))
    }
}
