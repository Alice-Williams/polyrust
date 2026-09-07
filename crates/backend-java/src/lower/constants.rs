//! Java lowering: constants.

use super::{Lowering, diagnostic};
use crate::ast::{JavaExpr, JavaType};
use crate::capabilities::{
    JavaConstantsInput, JavaConstantsNode, JavaListInput, JavaOptionInput, JavaRecordsInput,
    JavaResultInput,
};
use portable_build::CapabilityMapping;
use portable_codegen::GeneratedTypeId;
use portable_core_ir::{CoreConstantExpr, CoreConstantExprKind, CoreType, CoreTypeId};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn constant_expr(
        &self,
        value: &CoreConstantExpr,
        expected: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let ty = self.ty(expected)?;
        match &value.kind {
            CoreConstantExprKind::Literal(value) => self.value(value, expected),
            CoreConstantExprKind::Constant(id) => match self
                .features
                .mapping_for::<portable_build::Constants>()
                .lower(
                    &mut (),
                    JavaConstantsInput::Reference {
                        symbol: self.constants[id],
                        result: ty,
                    },
                )? {
                JavaConstantsNode::Expression(value) => Ok(value),
                JavaConstantsNode::Declaration(_) => Err(vec![diagnostic(
                    "Java Constants mapping returned a declaration for a reference",
                )]),
            },
            CoreConstantExprKind::Record { record, fields } => {
                let expressions = fields
                    .iter()
                    .map(|field| {
                        let metadata = self.core.field(field.field).expect("verified field");
                        self.constant_expr(&field.value, metadata.ty)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.construct_java_values(self.records[record], &expressions, ty)
            }
            CoreConstantExprKind::Enum {
                enumeration,
                variant,
                fields,
            } if self.enum_is_payload_free(*enumeration) => {
                if !fields.is_empty() {
                    return Err(vec![diagnostic(
                        "payload-free Java enum constant cannot contain fields",
                    )]);
                }
                self.enum_variant_expr(*enumeration, *variant)
            }
            CoreConstantExprKind::Enum {
                variant, fields, ..
            } => {
                let expressions = fields
                    .iter()
                    .map(|field| {
                        let metadata = self.core.field(field.field).expect("verified field");
                        self.constant_expr(&field.value, metadata.ty)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.construct_java_values(self.variants[variant], &expressions, ty)
            }
            CoreConstantExprKind::Some(value) => {
                let CoreType::Option(inner) =
                    self.core.types().get(expected).expect("verified option")
                else {
                    return Err(vec![diagnostic("constant some has wrong type")]);
                };
                self.value_expression(
                    self.features
                        .mapping_for::<portable_build::OptionValues>()
                        .lower(
                            &mut (),
                            JavaOptionInput::Some {
                                value: Box::new(self.constant_expr(value, *inner)?),
                                result: ty,
                            },
                        )?,
                    "OptionValues constant construction",
                )
            }
            CoreConstantExprKind::None { .. } => self.value_expression(
                self.features
                    .mapping_for::<portable_build::OptionValues>()
                    .lower(&mut (), JavaOptionInput::None { result: ty })?,
                "OptionValues constant construction",
            ),
            CoreConstantExprKind::Ok { value, .. } => {
                let CoreType::Result { ok, .. } =
                    self.core.types().get(expected).expect("verified result")
                else {
                    return Err(vec![diagnostic("constant ok has wrong type")]);
                };
                self.value_expression(
                    self.features
                        .mapping_for::<portable_build::ResultValues>()
                        .lower(
                            &mut (),
                            JavaResultInput::Ok {
                                value: self.constant_expr(value, *ok)?,
                                result: ty,
                            },
                        )?,
                    "ResultValues constant construction",
                )
            }
            CoreConstantExprKind::Err { value, .. } => {
                let CoreType::Result { error, .. } =
                    self.core.types().get(expected).expect("verified result")
                else {
                    return Err(vec![diagnostic("constant error has wrong type")]);
                };
                self.value_expression(
                    self.features
                        .mapping_for::<portable_build::ResultValues>()
                        .lower(
                            &mut (),
                            JavaResultInput::Err {
                                value: self.constant_expr(value, *error)?,
                                result: ty,
                            },
                        )?,
                    "ResultValues constant construction",
                )
            }
            CoreConstantExprKind::List { element, elements } => {
                let values = elements
                    .iter()
                    .map(|value| self.constant_expr(value, *element))
                    .collect::<Result<Vec<_>, _>>()?;
                self.value_expression(
                    self.features
                        .mapping_for::<portable_build::ListValues>()
                        .lower(
                            &mut (),
                            JavaListInput::Value {
                                elements: values,
                                result: ty,
                            },
                        )?,
                    "ListValues constant construction",
                )
            }
            CoreConstantExprKind::Intrinsic(value) => self.constant_intrinsic(value, expected),
        }
    }

    fn construct_java_values(
        &self,
        owner: GeneratedTypeId,
        fields: &[JavaExpr],
        result_type: JavaType,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        self.lower_record_expr(JavaRecordsInput::Construction {
            owner,
            arguments: fields.to_vec(),
            result: result_type,
        })
    }
}
