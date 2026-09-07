//! Java lowering: values.

use super::{ExprPlan, Lowering, diagnostic};
use crate::ast::{JavaExpr, JavaType};
use crate::capabilities::{
    JavaBoolValuesInput, JavaBytesInput, JavaCharValuesInput, JavaEnumsInput, JavaF64ValuesInput,
    JavaI32ValuesInput, JavaI64ValuesInput, JavaListInput, JavaOptionInput, JavaRecordsInput,
    JavaResultInput, JavaTextValuesInput, JavaUnitValuesInput, JavaValueNode,
};
use portable_build::CapabilityMapping;
use portable_codegen::GeneratedTypeId;
use portable_core_ir::{CoreExprField, CoreType, CoreTypeId, CoreValue, CoreValueField};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn construct_generated_plan(
        &self,
        owner: GeneratedTypeId,
        fields: &[CoreExprField],
        result_type: JavaType,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let ids = fields.iter().map(|field| field.value).collect::<Vec<_>>();
        let (statements, arguments) = self.expr_list(&ids, callable_return)?;
        let value = self.lower_record_expr(JavaRecordsInput::Construction {
            owner,
            arguments,
            result: result_type,
        })?;
        Ok(ExprPlan { statements, value })
    }

    pub(super) fn lower_unit_value(&self) -> Result<JavaExpr, Vec<Diagnostic>> {
        self.value_expression(
            self.features
                .mapping_for::<portable_build::UnitValues>()
                .lower(&mut (), JavaUnitValuesInput::Value)?,
            "UnitValues construction",
        )
    }

    pub(super) fn value_expression(
        &self,
        node: JavaValueNode,
        capability: &str,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        match node {
            JavaValueNode::Expression(value) => Ok(*value),
            JavaValueNode::Type(_) => Err(vec![diagnostic(&format!(
                "Java {capability} mapping returned a type for a value"
            ))]),
        }
    }

    pub(super) fn value_type(
        &self,
        node: JavaValueNode,
        capability: &str,
    ) -> Result<JavaType, Vec<Diagnostic>> {
        match node {
            JavaValueNode::Type(value) => Ok(value),
            JavaValueNode::Expression(_) => Err(vec![diagnostic(&format!(
                "Java {capability} mapping returned a value for a type"
            ))]),
        }
    }

    pub(super) fn value(
        &self,
        value: &CoreValue,
        expected: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let ty = self.ty(expected)?;
        match value {
            CoreValue::Unit => self.lower_unit_value(),
            CoreValue::Bool(value) => self.value_expression(
                self.features
                    .mapping_for::<portable_build::BoolValues>()
                    .lower(&mut (), JavaBoolValuesInput::Value(*value))?,
                "BoolValues construction",
            ),
            CoreValue::I32(value) => self.value_expression(
                self.features
                    .mapping_for::<portable_build::I32Values>()
                    .lower(&mut (), JavaI32ValuesInput::Value(*value))?,
                "I32Values construction",
            ),
            CoreValue::I64(value) => self.value_expression(
                self.features
                    .mapping_for::<portable_build::I64Values>()
                    .lower(&mut (), JavaI64ValuesInput::Value(*value))?,
                "I64Values construction",
            ),
            CoreValue::F64(value) => self.value_expression(
                self.features
                    .mapping_for::<portable_build::F64Values>()
                    .lower(&mut (), JavaF64ValuesInput::Value(value.0))?,
                "F64Values construction",
            ),
            CoreValue::Char(value) => self.value_expression(
                self.features
                    .mapping_for::<portable_build::CharValues>()
                    .lower(&mut (), JavaCharValuesInput::Value(*value))?,
                "CharValues construction",
            ),
            CoreValue::String(value) => self.value_expression(
                self.features
                    .mapping_for::<portable_build::TextValues>()
                    .lower(&mut (), JavaTextValuesInput::Value(value.clone()))?,
                "TextValues construction",
            ),
            CoreValue::Bytes(values) => self.value_expression(
                self.features
                    .mapping_for::<portable_build::BytesValues>()
                    .lower(
                        &mut (),
                        JavaBytesInput::Value {
                            values: values.clone(),
                            result: ty,
                        },
                    )?,
                "BytesValues construction",
            ),
            CoreValue::List(values) => {
                let CoreType::List(element) =
                    self.core.types().get(expected).expect("verified list type")
                else {
                    return Err(vec![diagnostic("list value does not have a list type")]);
                };
                let elements = values
                    .iter()
                    .map(|value| self.value(value, *element))
                    .collect::<Result<Vec<_>, _>>()?;
                self.value_expression(
                    self.features
                        .mapping_for::<portable_build::ListValues>()
                        .lower(
                            &mut (),
                            JavaListInput::Value {
                                elements,
                                result: ty,
                            },
                        )?,
                    "ListValues construction",
                )
            }
            CoreValue::None => self.value_expression(
                self.features
                    .mapping_for::<portable_build::OptionValues>()
                    .lower(&mut (), JavaOptionInput::None { result: ty })?,
                "OptionValues construction",
            ),
            CoreValue::Some(value) => {
                let CoreType::Option(inner) = self
                    .core
                    .types()
                    .get(expected)
                    .expect("verified option type")
                else {
                    return Err(vec![diagnostic("some value does not have an option type")]);
                };
                self.value_expression(
                    self.features
                        .mapping_for::<portable_build::OptionValues>()
                        .lower(
                            &mut (),
                            JavaOptionInput::Some {
                                value: Box::new(self.value(value, *inner)?),
                                result: ty,
                            },
                        )?,
                    "OptionValues construction",
                )
            }
            CoreValue::Ok(value) => {
                let CoreType::Result { ok, .. } = self
                    .core
                    .types()
                    .get(expected)
                    .expect("verified result type")
                else {
                    return Err(vec![diagnostic("ok value does not have a result type")]);
                };
                self.value_expression(
                    self.features
                        .mapping_for::<portable_build::ResultValues>()
                        .lower(
                            &mut (),
                            JavaResultInput::Ok {
                                value: self.value(value, *ok)?,
                                result: ty,
                            },
                        )?,
                    "ResultValues construction",
                )
            }
            CoreValue::Err(value) => {
                let CoreType::Result { error, .. } = self
                    .core
                    .types()
                    .get(expected)
                    .expect("verified result type")
                else {
                    return Err(vec![diagnostic("error value does not have a result type")]);
                };
                self.value_expression(
                    self.features
                        .mapping_for::<portable_build::ResultValues>()
                        .lower(
                            &mut (),
                            JavaResultInput::Err {
                                value: self.value(value, *error)?,
                                result: ty,
                            },
                        )?,
                    "ResultValues construction",
                )
            }
            CoreValue::Record { record, fields } => {
                self.lower_record_expr(JavaRecordsInput::Construction {
                    owner: self.records[record],
                    arguments: self.value_arguments(fields)?,
                    result: ty,
                })
            }
            CoreValue::Enum {
                enumeration,
                variant,
                fields,
            } if self.enum_is_payload_free(*enumeration) => {
                if !fields.is_empty() {
                    return Err(vec![diagnostic(
                        "payload-free Java enum value cannot contain fields",
                    )]);
                }
                self.enum_variant_expr(*enumeration, *variant)
            }
            CoreValue::Enum {
                variant, fields, ..
            } => self.lower_enum_expr(JavaEnumsInput::PayloadConstruction {
                variant: self.variants[variant],
                arguments: self.value_arguments(fields)?,
                result: ty,
            }),
        }
    }

    fn value_arguments(&self, fields: &[CoreValueField]) -> Result<Vec<JavaExpr>, Vec<Diagnostic>> {
        fields
            .iter()
            .map(|field| {
                let metadata = self.core.field(field.field).expect("verified field");
                self.value(&field.value, metadata.ty)
            })
            .collect()
    }
}
