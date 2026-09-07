//! Public API boundary normalization and tagged-value factories.

use super::call_builders::{known_generic_call, member_call, new_known, runtime_call};
use super::declaration_builders::{identifier, length_prefixed_type, public_factory_method};
use super::{ExprPlan, Lowering, diagnostic};
use crate::ast::{
    JavaBlock, JavaExpr, JavaKnownType, JavaLocalFinality, JavaMember, JavaMemberOrigin,
    JavaParameter, JavaPrimitive, JavaStmt, JavaType, JavaTypeName,
};
use crate::capabilities::{JavaOptionInput, JavaResultInput};
use crate::dialect::{
    JavaKnownCallable, JavaKnownConstructor, JavaKnownMethod, JavaRuntimeCallable,
};
use portable_build::CapabilityMapping;
use portable_core_ir::{CoreType, CoreTypeId};
use portable_diagnostics::Diagnostic;

#[cfg(test)]
#[path = "../tests/boundary_mapping.rs"]
mod tests;

enum ResultArm {
    Ok,
    Error,
}

impl Lowering<'_> {
    fn option_value(&self, input: JavaOptionInput) -> Result<JavaExpr, Vec<Diagnostic>> {
        self.value_expression(
            self.features
                .mapping_for::<portable_build::OptionValues>()
                .lower(&mut (), input)?,
            "OptionValues boundary construction",
        )
    }

    fn result_value(&self, input: JavaResultInput) -> Result<JavaExpr, Vec<Diagnostic>> {
        self.value_expression(
            self.features
                .mapping_for::<portable_build::ResultValues>()
                .lower(&mut (), input)?,
            "ResultValues boundary construction",
        )
    }
    pub(super) fn public_tagged_value_factories(&self) -> Result<Vec<JavaMember>, Vec<Diagnostic>> {
        let mut members = Vec::new();
        for (id, kind) in self.core.types().iter() {
            match kind {
                CoreType::Option(inner) => {
                    let suffix = self.factory_type_suffix(id)?;
                    let option_type = self.ty(id)?;
                    members.push(public_factory_method(
                        &format!("__polyrust_noneOf{suffix}"),
                        option_type.clone(),
                        vec![],
                        ExprPlan::pure(self.option_value(JavaOptionInput::None {
                            result: option_type.clone(),
                        })?),
                    ));
                    let payload_type = self.ty(*inner)?;
                    let input = JavaExpr::local(payload_type.clone(), identifier("value"));
                    let normalized = self.normalize_boundary_value(*inner, input)?;
                    members.push(public_factory_method(
                        &format!("__polyrust_someOf{suffix}"),
                        option_type.clone(),
                        vec![JavaParameter {
                            ty: payload_type,
                            name: identifier("value"),
                            final_parameter: true,
                        }],
                        ExprPlan {
                            statements: normalized.statements,
                            value: self.option_value(JavaOptionInput::Some {
                                value: Box::new(normalized.value),
                                result: option_type,
                            })?,
                        },
                    ));
                }
                CoreType::Result { ok, error } => {
                    let suffix = self.factory_type_suffix(id)?;
                    let result_type = self.ty(id)?;
                    for (label, payload, arm) in [
                        ("ok", *ok, ResultArm::Ok),
                        ("error", *error, ResultArm::Error),
                    ] {
                        let payload_type = self.ty(payload)?;
                        let input = JavaExpr::local(payload_type.clone(), identifier("value"));
                        let normalized = self.normalize_boundary_value(payload, input)?;
                        members.push(public_factory_method(
                            &format!("__polyrust_{label}Of{suffix}"),
                            result_type.clone(),
                            vec![JavaParameter {
                                ty: payload_type,
                                name: identifier("value"),
                                final_parameter: true,
                            }],
                            ExprPlan {
                                statements: normalized.statements,
                                value: self.result_value(match arm {
                                    ResultArm::Ok => JavaResultInput::Ok {
                                        value: normalized.value,
                                        result: result_type.clone(),
                                    },
                                    ResultArm::Error => JavaResultInput::Err {
                                        value: normalized.value,
                                        result: result_type.clone(),
                                    },
                                })?,
                            },
                        ));
                    }
                }
                CoreType::Unit
                | CoreType::Bool
                | CoreType::I32
                | CoreType::I64
                | CoreType::F64
                | CoreType::Char
                | CoreType::String
                | CoreType::Bytes
                | CoreType::List(_)
                | CoreType::Record(_)
                | CoreType::Enum(_)
                | CoreType::Interface(_) => {}
            }
        }
        Ok(members)
    }

    fn factory_type_suffix(&self, id: CoreTypeId) -> Result<String, Vec<Diagnostic>> {
        let Some(kind) = self.core.types().get(id) else {
            return Err(vec![diagnostic(
                "missing CoreIR type for Java value factory",
            )]);
        };
        let suffix = match kind {
            CoreType::Unit => "Unit".to_owned(),
            CoreType::Bool => "Bool".to_owned(),
            CoreType::I32 => "I32".to_owned(),
            CoreType::I64 => "I64".to_owned(),
            CoreType::F64 => "F64".to_owned(),
            CoreType::Char => "Char".to_owned(),
            CoreType::String => "String".to_owned(),
            CoreType::Bytes => "Bytes".to_owned(),
            CoreType::List(element) => {
                length_prefixed_type("List", &self.factory_type_suffix(*element)?)
            }
            CoreType::Option(inner) => {
                length_prefixed_type("Option", &self.factory_type_suffix(*inner)?)
            }
            CoreType::Result { ok, error } => {
                let ok = self.factory_type_suffix(*ok)?;
                let error = self.factory_type_suffix(*error)?;
                format!("Result{}_{}{}_{}", ok.len(), ok, error.len(), error)
            }
            CoreType::Record(record) => {
                let name = identifier(
                    &self
                        .core
                        .record(*record)
                        .expect("verified record")
                        .header
                        .name,
                );
                length_prefixed_type("Record", name.as_str())
            }
            CoreType::Enum(enumeration) => {
                let name = identifier(
                    &self
                        .core
                        .enumeration(*enumeration)
                        .expect("verified enum")
                        .header
                        .name,
                );
                length_prefixed_type("Enum", name.as_str())
            }
            CoreType::Interface(interface) => {
                let name = identifier(
                    &self
                        .core
                        .interface(*interface)
                        .expect("verified interface")
                        .header
                        .name,
                );
                length_prefixed_type("Interface", name.as_str())
            }
        };
        Ok(suffix)
    }

    pub(super) fn normalize_boundary_value(
        &self,
        core_type: CoreTypeId,
        input: JavaExpr,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let Some(kind) = self.core.types().get(core_type) else {
            return Err(vec![diagnostic("missing boundary CoreIR type")]);
        };
        match kind {
            CoreType::Unit
            | CoreType::Char
            | CoreType::Bytes
            | CoreType::Record(_)
            | CoreType::Enum(_)
            | CoreType::Interface(_) => Ok(ExprPlan::pure(known_generic_call(
                JavaKnownCallable::ObjectsRequireNonNull,
                vec![input.clone()],
                input.ty,
            ))),
            CoreType::String => Ok(ExprPlan::pure(runtime_call(
                JavaRuntimeCallable::RequireScalarString,
                vec![input.clone()],
                input.ty,
            ))),
            CoreType::Bool | CoreType::I32 | CoreType::I64 | CoreType::F64 => {
                Ok(ExprPlan::pure(input))
            }
            CoreType::List(element) => self.normalize_boundary_list(*element, input),
            CoreType::Option(inner) => self.normalize_boundary_option(*inner, input),
            CoreType::Result { ok, error } => self.normalize_boundary_result(*ok, *error, input),
        }
    }

    fn normalize_boundary_list(
        &self,
        element: CoreTypeId,
        input: JavaExpr,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let list_type = input.ty.clone();
        let element_type = self.ty(element)?.boxed();
        let mutable_type = JavaType::Generic {
            raw: JavaTypeName::Known(JavaKnownType::ArrayList),
            arguments: vec![element_type.clone()],
        };
        let (output_name, output) = self.temporary("boundaryList", mutable_type.clone());
        let (item_name, item) = self.temporary("boundaryItem", element_type.clone());
        let normalized = self.normalize_boundary_value(element, item)?;
        let mut loop_body = normalized.statements;
        loop_body.push(JavaStmt::Expression(member_call(
            output.clone(),
            JavaKnownMethod::ArrayListAdd.name().text(),
            vec![normalized.value],
            JavaType::primitive(JavaPrimitive::Boolean),
            JavaMemberOrigin::Known(JavaKnownMethod::ArrayListAdd),
        )));
        Ok(ExprPlan {
            statements: vec![
                JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: mutable_type.clone(),
                    name: output_name,
                    value: Some(new_known(
                        JavaKnownConstructor::ArrayList,
                        mutable_type,
                        vec![],
                    )),
                },
                JavaStmt::ForEach {
                    binding_type: element_type,
                    binding: item_name,
                    iterable: input,
                    body: JavaBlock::new(loop_body),
                },
            ],
            value: known_generic_call(JavaKnownCallable::ListCopyOf, vec![output], list_type),
        })
    }

    fn normalize_boundary_option(
        &self,
        inner: CoreTypeId,
        input: JavaExpr,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let option_type = input.ty.clone();
        let (result_name, result) = self.temporary("boundaryOption", option_type.clone());
        let payload = runtime_call(
            JavaRuntimeCallable::OptionValue,
            vec![input.clone()],
            self.ty(inner)?.boxed(),
        );
        let normalized = self.normalize_boundary_value(inner, payload)?;
        let mut some_statements = normalized.statements;
        some_statements.push(JavaStmt::Assign {
            target: result.clone(),
            value: self.option_value(JavaOptionInput::Some {
                value: Box::new(normalized.value),
                result: option_type.clone(),
            })?,
        });
        Ok(ExprPlan {
            statements: vec![
                JavaStmt::Local {
                    finality: JavaLocalFinality::Mutable,
                    ty: option_type.clone(),
                    name: result_name,
                    value: None,
                },
                JavaStmt::If {
                    condition: runtime_call(
                        JavaRuntimeCallable::OptionIsSome,
                        vec![input],
                        JavaType::primitive(JavaPrimitive::Boolean),
                    ),
                    then_block: JavaBlock::new(some_statements),
                    else_block: Some(JavaBlock::new(vec![JavaStmt::Assign {
                        target: result.clone(),
                        value: self.option_value(JavaOptionInput::None {
                            result: option_type,
                        })?,
                    }])),
                },
            ],
            value: result,
        })
    }

    fn normalize_boundary_result(
        &self,
        ok: CoreTypeId,
        error: CoreTypeId,
        input: JavaExpr,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let result_type = input.ty.clone();
        let (result_name, result) = self.temporary("boundaryResult", result_type.clone());
        let ok_value = runtime_call(
            JavaRuntimeCallable::ValueResultValue,
            vec![input.clone()],
            self.ty(ok)?.boxed(),
        );
        let normalized_ok = self.normalize_boundary_value(ok, ok_value)?;
        let mut ok_statements = normalized_ok.statements;
        ok_statements.push(JavaStmt::Assign {
            target: result.clone(),
            value: self.result_value(JavaResultInput::Ok {
                value: normalized_ok.value,
                result: result_type.clone(),
            })?,
        });
        let error_value = runtime_call(
            JavaRuntimeCallable::ValueResultError,
            vec![input.clone()],
            self.ty(error)?.boxed(),
        );
        let normalized_error = self.normalize_boundary_value(error, error_value)?;
        let mut error_statements = normalized_error.statements;
        error_statements.push(JavaStmt::Assign {
            target: result.clone(),
            value: self.result_value(JavaResultInput::Err {
                value: normalized_error.value,
                result: result_type.clone(),
            })?,
        });
        Ok(ExprPlan {
            statements: vec![
                JavaStmt::Local {
                    finality: JavaLocalFinality::Mutable,
                    ty: result_type,
                    name: result_name,
                    value: None,
                },
                JavaStmt::If {
                    condition: runtime_call(
                        JavaRuntimeCallable::ValueResultIsOk,
                        vec![input],
                        JavaType::primitive(JavaPrimitive::Boolean),
                    ),
                    then_block: JavaBlock::new(ok_statements),
                    else_block: Some(JavaBlock::new(error_statements)),
                },
            ],
            value: result,
        })
    }
}
