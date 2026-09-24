//! File, statement and initializer dependency traversal.

use super::{CFileDependencies, CTagDependency, CTypeRequirement};
use crate::ast::{
    CAggregateRef, CBlock, CCaseConstant, CDeclarationKind, CDefinitionKind, CFileItem,
    CInitializer, CInitializerKind, CLiteral, CStatement, CStatementKind,
};

impl CFileDependencies {
    pub(super) fn item(&mut self, item: &CFileItem) {
        match item {
            CFileItem::Comment(_) => {}
            CFileItem::StaticAssert(assertion) => self.value(assertion.condition()),
            CFileItem::Declaration(declaration) => match declaration.kind() {
                CDeclarationKind::ForwardTag(_) | CDeclarationKind::Enum { .. } => {}
                CDeclarationKind::Typedef(alias) => {
                    self.object_type(alias.target(), CTypeRequirement::Declaration)
                }
                CDeclarationKind::Aggregate { members, .. } => {
                    for member in members {
                        self.object_type(member.ty(), CTypeRequirement::Complete);
                    }
                }
                CDeclarationKind::FunctionPrototype { function, .. } => {
                    self.signature(function.signature(), CTypeRequirement::Declaration)
                }
                CDeclarationKind::ObjectDeclaration(object) => {
                    self.object_type(object.ty(), CTypeRequirement::Declaration)
                }
            },
            CFileItem::Definition(definition) => match definition.kind() {
                CDefinitionKind::Function {
                    function,
                    parameters,
                    body,
                    ..
                } => {
                    self.signature(function.signature(), CTypeRequirement::Complete);
                    for parameter in parameters {
                        self.object_type(parameter.ty(), CTypeRequirement::Complete);
                    }
                    self.block(body);
                }
                CDefinitionKind::Object {
                    object,
                    initializer,
                    ..
                } => {
                    self.object_type(object.ty(), CTypeRequirement::Complete);
                    self.initializer(initializer);
                }
            },
        }
    }

    fn initializer(&mut self, initializer: &CInitializer) {
        self.object_type(initializer.ty(), CTypeRequirement::Complete);
        match initializer.kind() {
            CInitializerKind::Expression(value) => self.value(value),
            CInitializerKind::Zero(ty) => self.object_type(ty, CTypeRequirement::Complete),
            CInitializerKind::Array {
                declared_type,
                elements,
            } => {
                self.object_type(declared_type, CTypeRequirement::Complete);
                for element in elements {
                    self.initializer(element);
                }
            }
            CInitializerKind::Struct { owner, members } => {
                self.aggregate(&CAggregateRef::Struct(owner.clone()));
                for (member, value) in members {
                    self.members.insert(member.clone());
                    self.object_type(member.ty(), CTypeRequirement::Complete);
                    self.initializer(value);
                }
            }
            CInitializerKind::Union {
                owner,
                member,
                value,
            } => {
                self.aggregate(&CAggregateRef::Union(owner.clone()));
                self.members.insert(member.clone());
                self.object_type(member.ty(), CTypeRequirement::Complete);
                self.initializer(value);
            }
        }
    }

    fn block(&mut self, block: &CBlock) {
        for statement in block.statements() {
            self.statement(statement);
        }
    }

    fn statement(&mut self, statement: &CStatement) {
        match statement.kind() {
            CStatementKind::Empty
            | CStatementKind::Break(_)
            | CStatementKind::Continue(_)
            | CStatementKind::CleanupJump(_) => {}
            CStatementKind::Block(block) => self.block(block),
            CStatementKind::Declare(declaration) => {
                self.object_type(declaration.local().ty(), CTypeRequirement::Complete);
                if let Some(value) = declaration.initializer() {
                    self.initializer(value);
                }
            }
            CStatementKind::Assign { place, value } => {
                self.object_type(place.ty(), CTypeRequirement::Complete);
                self.place(place);
                self.value(value);
            }
            CStatementKind::Evaluate(effect) => self.call(effect.call()),
            CStatementKind::Discard(value) => self.value(value),
            CStatementKind::If {
                condition,
                then_block,
                else_block,
            } => {
                self.value(condition);
                self.block(then_block);
                self.block(else_block);
            }
            CStatementKind::BoundedLoop {
                progress,
                condition,
                body,
                ..
            } => {
                self.object_type(progress.counter().ty(), CTypeRequirement::Complete);
                self.object_type(progress.bound().ty(), CTypeRequirement::Complete);
                self.value(condition);
                self.block(body);
            }
            CStatementKind::Switch {
                value,
                arms,
                default,
                ..
            } => {
                self.value(value);
                for arm in arms {
                    for case in arm.cases() {
                        match case {
                            CCaseConstant::Signed(value) => self.literal(&CLiteral::Signed(*value)),
                            CCaseConstant::Unsigned(value) => {
                                self.literal(&CLiteral::Unsigned(*value))
                            }
                            CCaseConstant::Enumerator(value) => self.tag(
                                CTagDependency::Enum(value.owner().clone()),
                                CTypeRequirement::Complete,
                            ),
                        }
                    }
                    self.block(arm.body());
                }
                self.block(default);
            }
            CStatementKind::Return(value) => {
                if let Some(value) = value {
                    self.value(value);
                }
            }
            CStatementKind::Label { statement, .. } => self.statement(statement),
        }
    }
}
