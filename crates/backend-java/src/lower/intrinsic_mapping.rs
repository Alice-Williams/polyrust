//! Java lowering: intrinsic mapping.

use super::{JavaIntrinsicExpr, Lowering, diagnostic};
use crate::ast::{JavaExpr, JavaType, JavaTypeName};
use crate::capabilities::{
    JavaEnumEqualityOperator, JavaEnumsInput, JavaEnumsNode, JavaIntrinsicFamily,
    classify_intrinsic,
};
use portable_build::{
    BytesOperations, CapabilityMapping, CheckedIntegerArithmetic, CheckedIntegerShifts, Equality,
    FloatingPointArithmetic, FloatingPointInspection, IntegerBitwise, IntegerConversions,
    ListOperations, OptionOperations, Ordering, ResultOperations, StringConcatenation,
    StringInspection, StringTransformation, Utf8Conversions, WrappingIntegerArithmetic,
};
use portable_codegen::GeneratedTypeId;
use portable_core_ir::{CoreBinaryIntrinsic, CoreIntrinsicExpr};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn intrinsic_java(
        &self,
        value: CoreIntrinsicExpr<JavaExpr>,
        result: JavaType,
    ) -> Result<JavaIntrinsicExpr, Vec<Diagnostic>> {
        if let CoreIntrinsicExpr::Binary {
            operation,
            left,
            right,
        } = &value
            && matches!(
                operation,
                CoreBinaryIntrinsic::Equal | CoreBinaryIntrinsic::NotEqual
            )
            && let Some(enumeration) = self.payload_free_java_enum_type(&left.ty)
        {
            let JavaEnumsNode::Expression(equal) =
                self.features.mapping_for::<portable_build::Enums>().lower(
                    &mut (),
                    JavaEnumsInput::Equality {
                        operator: match operation {
                            CoreBinaryIntrinsic::Equal => JavaEnumEqualityOperator::Equal,
                            CoreBinaryIntrinsic::NotEqual => JavaEnumEqualityOperator::NotEqual,
                            _ => {
                                unreachable!("enum equality branch accepts only equality operators")
                            }
                        },
                        enumeration,
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    },
                )?
            else {
                return Err(vec![diagnostic(
                    "Java Enums mapping returned a non-expression for equality",
                )]);
            };
            return Ok(JavaIntrinsicExpr::Infallible(*equal));
        }
        let mut context = ();
        match classify_intrinsic(value, result)? {
            JavaIntrinsicFamily::Equality(input) => self
                .features
                .mapping_for::<Equality>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::Ordering(input) => self
                .features
                .mapping_for::<Ordering>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::CheckedIntegerArithmetic(input) => self
                .features
                .mapping_for::<CheckedIntegerArithmetic>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::WrappingIntegerArithmetic(input) => self
                .features
                .mapping_for::<WrappingIntegerArithmetic>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::FloatingPointArithmetic(input) => self
                .features
                .mapping_for::<FloatingPointArithmetic>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::StringConcatenation(input) => self
                .features
                .mapping_for::<StringConcatenation>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::IntegerBitwise(input) => self
                .features
                .mapping_for::<IntegerBitwise>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::CheckedIntegerShifts(input) => self
                .features
                .mapping_for::<CheckedIntegerShifts>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::FloatingPointInspection(input) => self
                .features
                .mapping_for::<FloatingPointInspection>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::StringInspection(input) => self
                .features
                .mapping_for::<StringInspection>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::StringTransformation(input) => self
                .features
                .mapping_for::<StringTransformation>()
                .lower(&mut context, *input),
            JavaIntrinsicFamily::BytesOperations(input) => self
                .features
                .mapping_for::<BytesOperations>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::ListOperations(input) => self
                .features
                .mapping_for::<ListOperations>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::OptionOperations(input) => self
                .features
                .mapping_for::<OptionOperations>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::ResultOperations(input) => self
                .features
                .mapping_for::<ResultOperations>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::IntegerConversions(input) => self
                .features
                .mapping_for::<IntegerConversions>()
                .lower(&mut context, input),
            JavaIntrinsicFamily::Utf8Conversions(input) => self
                .features
                .mapping_for::<Utf8Conversions>()
                .lower(&mut context, input),
        }
    }

    fn payload_free_java_enum_type(&self, ty: &JavaType) -> Option<GeneratedTypeId> {
        let JavaType::Reference(JavaTypeName::Generated(generated)) = ty else {
            return None;
        };
        self.enums.iter().find_map(|(enumeration, candidate)| {
            (candidate == generated && self.enum_is_payload_free(*enumeration))
                .then_some(*generated)
        })
    }
}
