//! Java lowering: enums.

use super::{Lowering, diagnostic};
use crate::ast::{
    JavaExpr, JavaMember, JavaRecordComponent, JavaRecordComponentOrigin, JavaRuntimeMember,
    JavaTypeDeclaration,
};
use crate::capabilities::{
    JavaEnumPayloadVariantInput, JavaEnumVariantInput, JavaEnumsInput, JavaEnumsNode,
};
use crate::dialect::JavaRuntimeCallable;
use portable_build::CapabilityMapping;
use portable_core_ir::{CoreEnumId, CoreVariantId};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn enum_declarations(
        &self,
        id: CoreEnumId,
    ) -> Result<Vec<JavaTypeDeclaration>, Vec<Diagnostic>> {
        let enumeration = self.core.enumeration(id).expect("verified enum");
        if self.enum_is_payload_free(id) {
            return match self.features.mapping_for::<portable_build::Enums>().lower(
                &mut (),
                JavaEnumsInput::Declaration {
                    declared: self.enums[&id],
                    visibility: enumeration.header.visibility,
                    name: self.names.enumeration(id).as_str().to_owned(),
                    variants: enumeration
                        .variants
                        .iter()
                        .map(|variant_id| JavaEnumVariantInput {
                            declared: self.enum_values[variant_id],
                            name: self.names.enum_value(*variant_id).as_str().to_owned(),
                        })
                        .collect(),
                },
            )? {
                JavaEnumsNode::Declaration(declarations) => Ok(declarations),
                JavaEnumsNode::Type(_)
                | JavaEnumsNode::Expression(_)
                | JavaEnumsNode::Statement(_) => Err(vec![diagnostic(
                    "Java Enums mapping returned a value for a declaration",
                )]),
            };
        }
        let variants = enumeration
            .variants
            .iter()
            .map(|variant_id| {
                let variant = self.core.variant(*variant_id).expect("verified variant");
                let variant_type = self.variants[variant_id];
                let name = self.names.variant(*variant_id).as_str().to_owned();
                Ok(JavaEnumPayloadVariantInput {
                    declared: variant_type,
                    name: name.clone(),
                    components: variant
                        .fields
                        .iter()
                        .map(|field| {
                            let value = self.core.field(*field).expect("verified field");
                            Ok(JavaRecordComponent {
                                origin: JavaRecordComponentOrigin::Core(*field),
                                ty: self.ty(value.ty)?,
                                name: self.names.field(*field).clone(),
                            })
                        })
                        .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
                    members: vec![
                        JavaMember::Constructor(self.generated_record_constructor(
                            variant_type,
                            &name,
                            enumeration.header.visibility,
                            &variant.fields,
                        )?),
                        JavaMember::Method(self.value_equality_method(
                            variant_type,
                            &variant.fields,
                            JavaRuntimeCallable::SemanticEqual,
                            JavaRuntimeMember::SemanticEquals,
                        )?),
                        JavaMember::Method(self.value_equality_method(
                            variant_type,
                            &variant.fields,
                            JavaRuntimeCallable::DeepEqual,
                            JavaRuntimeMember::DeepEquals,
                        )?),
                    ],
                })
            })
            .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
        match self.features.mapping_for::<portable_build::Enums>().lower(
            &mut (),
            JavaEnumsInput::PayloadDeclaration {
                declared: self.enums[&id],
                visibility: enumeration.header.visibility,
                name: self.names.enumeration(id).as_str().to_owned(),
                variants,
            },
        )? {
            JavaEnumsNode::Declaration(declarations) => Ok(declarations),
            JavaEnumsNode::Type(_) | JavaEnumsNode::Expression(_) | JavaEnumsNode::Statement(_) => {
                Err(vec![diagnostic(
                    "Java Enums mapping returned a value for a declaration",
                )])
            }
        }
    }

    pub(super) fn enum_is_payload_free(&self, id: CoreEnumId) -> bool {
        crate::preflight::payload_free_enum(self.core, id)
    }

    pub(super) fn enum_variant_expr(
        &self,
        enumeration: CoreEnumId,
        variant: CoreVariantId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        self.lower_enum_expr(JavaEnumsInput::Variant {
            enumeration: self.enums[&enumeration],
            variant: self.enum_values[&variant],
        })
    }

    pub(super) fn lower_enum_expr(
        &self,
        input: JavaEnumsInput,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        match self
            .features
            .mapping_for::<portable_build::Enums>()
            .lower(&mut (), input)?
        {
            JavaEnumsNode::Expression(value) => Ok(*value),
            JavaEnumsNode::Type(_)
            | JavaEnumsNode::Declaration(_)
            | JavaEnumsNode::Statement(_) => Err(vec![diagnostic(
                "Java Enums mapping returned a non-expression for a value operation",
            )]),
        }
    }
}
