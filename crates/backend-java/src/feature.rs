use portable_build::*;
use portable_core_ir::{
    CoreBinaryIntrinsic, CoreIntrinsicExpr, CoreTernaryIntrinsic, CoreUnaryIntrinsic,
    CoreVariadicIntrinsic,
};
use portable_diagnostics::Diagnostic;

use crate::{
    ast::{JavaExpr, JavaMethod, JavaType, JavaTypeDeclaration},
    dialect::JavaDialect,
    lower::{JavaIntrinsicExpr, lower_intrinsic_expression},
};

pub type JavaCapabilitySlots = implemented_capability_slots!(
    JavaFunctions,
    JavaLocalReads,
    JavaFunctionCalls,
    JavaRecords,
    JavaRecordConstruction,
    JavaFieldAccess,
    JavaBoolValues,
    JavaI32Values,
    JavaI64Values,
    JavaF64Values,
    JavaTextValues,
    JavaBooleanLogic,
    JavaEquality,
    JavaOrdering,
    JavaCheckedIntegerArithmetic,
    JavaWrappingIntegerArithmetic,
    JavaFloatingPointArithmetic,
    JavaStringConcatenation,
    JavaCharValues,
    JavaBytesValues,
    JavaListValues,
    JavaOptionValues,
    JavaResultValues,
    JavaListConstruction,
    JavaOptionConstruction,
    JavaResultConstruction,
    JavaIntegerBitwise,
    JavaCheckedIntegerShifts,
    JavaFloatingPointInspection,
    JavaStringInspection,
    JavaStringTransformation,
    JavaBytesOperations,
    JavaListOperations,
    JavaOptionOperations,
    JavaResultInspection,
    JavaIntegerConversions,
    JavaUtf8Conversions,
);

pub type JavaCapabilitySet = LanguageCapabilityPlugin<JavaDialect, JavaCapabilitySlots>;

pub(crate) fn java_capabilities() -> JavaCapabilitySet {
    java_plugin_builder()
        .support(JavaFunctions)
        .support(JavaLocalReads)
        .support(JavaFunctionCalls)
        .support(JavaRecords)
        .support(JavaRecordConstruction)
        .support(JavaFieldAccess)
        .support(JavaBoolValues)
        .support(JavaI32Values)
        .support(JavaI64Values)
        .support(JavaF64Values)
        .support(JavaTextValues)
        .support(JavaBooleanLogic)
        .support(JavaEquality)
        .support(JavaOrdering)
        .support(JavaCheckedIntegerArithmetic)
        .support(JavaWrappingIntegerArithmetic)
        .support(JavaFloatingPointArithmetic)
        .support(JavaStringConcatenation)
        .support(JavaCharValues)
        .support(JavaBytesValues)
        .support(JavaListValues)
        .support(JavaOptionValues)
        .support(JavaResultValues)
        .support(JavaListConstruction)
        .support(JavaOptionConstruction)
        .support(JavaResultConstruction)
        .support(JavaIntegerBitwise)
        .support(JavaCheckedIntegerShifts)
        .support(JavaFloatingPointInspection)
        .support(JavaStringInspection)
        .support(JavaStringTransformation)
        .support(JavaBytesOperations)
        .support(JavaListOperations)
        .support(JavaOptionOperations)
        .support(JavaResultInspection)
        .support(JavaIntegerConversions)
        .support(JavaUtf8Conversions)
        .build()
}

mod sealed {
    pub trait JavaCapabilityMapping {}
}

/// A Java mapping admitted by the sealed Java plugin builder.
pub trait JavaCapabilityMapping:
    sealed::JavaCapabilityMapping + CapabilityMapping<JavaDialect> + Copy + Send + Sync
{
}

/// Consuming Java-specific wrapper which admits only sealed Java AST mappings.
///
/// A generic mapping whose output is not part of Java's checked AST cannot be
/// smuggled into this builder, even if it names a real portable feature:
///
/// ```compile_fail
/// use portable_backend_java::{dialect::JavaDialect, feature::java_plugin_builder};
/// use portable_build::{CapabilityMapping, I32Values};
/// struct SourceStringMapping;
/// impl CapabilityMapping<JavaDialect> for SourceStringMapping {
///     type Capability = I32Values;
///     type Context = ();
///     type Input = i32;
///     type Output = String;
///     type Error = ();
///     fn lower(&self, _: &mut (), value: i32) -> Result<String, ()> {
///         Ok(value.to_string())
///     }
/// }
/// let _ = java_plugin_builder().support(SourceStringMapping);
/// ```
pub struct JavaPluginBuilder<Slots = EmptyCapabilitySlots> {
    inner: LanguagePluginBuilder<JavaDialect, Slots>,
}

type JavaRegisteredSlots<Slots, M> = <Slots as ReplaceMissing<
    <<M as CapabilityMapping<JavaDialect>>::Capability as Capability>::Index,
    M,
>>::Output;

pub fn java_plugin_builder() -> JavaPluginBuilder {
    JavaPluginBuilder {
        inner: language_plugin(JavaDialect),
    }
}

impl<Slots> JavaPluginBuilder<Slots> {
    pub fn support<M>(self, mapping: M) -> JavaPluginBuilder<JavaRegisteredSlots<Slots, M>>
    where
        M: JavaCapabilityMapping,
        Slots: ReplaceMissing<<M::Capability as Capability>::Index, M>,
    {
        JavaPluginBuilder {
            inner: self.inner.support(mapping),
        }
    }

    pub fn build(self) -> LanguageCapabilityPlugin<JavaDialect, Slots> {
        self.inner.build()
    }
}

macro_rules! ast_mapping {
    ($mapping:ident, $feature:ty, $ast:ty) => {
        #[doc(hidden)]
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $mapping;

        impl sealed::JavaCapabilityMapping for $mapping {}
        impl JavaCapabilityMapping for $mapping {}

        impl CapabilityMapping<JavaDialect> for $mapping {
            type Capability = $feature;
            type Context = ();
            type Input = $ast;
            type Output = $ast;
            type Error = Vec<Diagnostic>;

            fn lower(
                &self,
                _context: &mut Self::Context,
                input: Self::Input,
            ) -> Result<Self::Output, Self::Error> {
                Ok(input)
            }
        }
    };
}

ast_mapping!(JavaFunctions, Functions, JavaMethod);
ast_mapping!(JavaLocalReads, LocalReads, JavaExpr);
ast_mapping!(JavaFunctionCalls, FunctionCalls, JavaExpr);
ast_mapping!(JavaRecords, Records, JavaTypeDeclaration);
ast_mapping!(JavaRecordConstruction, RecordConstruction, JavaExpr);
ast_mapping!(JavaFieldAccess, FieldAccess, JavaExpr);
ast_mapping!(JavaBoolValues, BoolValues, JavaExpr);
ast_mapping!(JavaI32Values, I32Values, JavaExpr);
ast_mapping!(JavaI64Values, I64Values, JavaExpr);
ast_mapping!(JavaF64Values, F64Values, JavaExpr);
ast_mapping!(JavaTextValues, TextValues, JavaExpr);
ast_mapping!(JavaCharValues, CharValues, JavaExpr);
ast_mapping!(JavaBytesValues, BytesValues, JavaExpr);
ast_mapping!(JavaListValues, ListValues, JavaExpr);
ast_mapping!(JavaOptionValues, OptionValues, JavaExpr);
ast_mapping!(JavaResultValues, ResultValues, JavaExpr);
ast_mapping!(JavaListConstruction, ListConstruction, JavaExpr);
ast_mapping!(JavaOptionConstruction, OptionConstruction, JavaExpr);
ast_mapping!(JavaResultConstruction, ResultConstruction, JavaExpr);

#[doc(hidden)]
pub struct JavaIntrinsicMappingInput<F: Capability> {
    value: CoreIntrinsicExpr<JavaExpr>,
    result: JavaType,
    feature: std::marker::PhantomData<F>,
}

impl<F: Capability> JavaIntrinsicMappingInput<F> {
    fn new(value: CoreIntrinsicExpr<JavaExpr>, result: JavaType) -> Self {
        Self {
            value,
            result,
            feature: std::marker::PhantomData,
        }
    }
}

macro_rules! intrinsic_mapping {
    ($mapping:ident, $feature:ty) => {
        #[doc(hidden)]
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $mapping;

        impl sealed::JavaCapabilityMapping for $mapping {}
        impl JavaCapabilityMapping for $mapping {}

        impl CapabilityMapping<JavaDialect> for $mapping {
            type Capability = $feature;
            type Context = ();
            type Input = JavaIntrinsicMappingInput<$feature>;
            type Output = JavaIntrinsicExpr;
            type Error = Vec<Diagnostic>;

            fn lower(
                &self,
                _context: &mut Self::Context,
                input: Self::Input,
            ) -> Result<Self::Output, Self::Error> {
                lower_intrinsic_expression(input.value, input.result)
            }
        }
    };
}

intrinsic_mapping!(JavaBooleanLogic, BooleanLogic);
intrinsic_mapping!(JavaEquality, Equality);
intrinsic_mapping!(JavaOrdering, Ordering);
intrinsic_mapping!(JavaCheckedIntegerArithmetic, CheckedIntegerArithmetic);
intrinsic_mapping!(JavaWrappingIntegerArithmetic, WrappingIntegerArithmetic);
intrinsic_mapping!(JavaFloatingPointArithmetic, FloatingPointArithmetic);
intrinsic_mapping!(JavaStringConcatenation, StringConcatenation);
intrinsic_mapping!(JavaIntegerBitwise, IntegerBitwise);
intrinsic_mapping!(JavaCheckedIntegerShifts, CheckedIntegerShifts);
intrinsic_mapping!(JavaFloatingPointInspection, FloatingPointInspection);
intrinsic_mapping!(JavaStringInspection, StringInspection);
intrinsic_mapping!(JavaStringTransformation, StringTransformation);
intrinsic_mapping!(JavaBytesOperations, BytesOperations);
intrinsic_mapping!(JavaListOperations, ListOperations);
intrinsic_mapping!(JavaOptionOperations, OptionOperations);
intrinsic_mapping!(JavaResultInspection, ResultInspection);
intrinsic_mapping!(JavaIntegerConversions, IntegerConversions);
intrinsic_mapping!(JavaUtf8Conversions, Utf8Conversions);

pub(crate) enum JavaIntrinsicFamily {
    BooleanLogic(JavaIntrinsicMappingInput<BooleanLogic>),
    Equality(JavaIntrinsicMappingInput<Equality>),
    Ordering(JavaIntrinsicMappingInput<Ordering>),
    CheckedIntegerArithmetic(JavaIntrinsicMappingInput<CheckedIntegerArithmetic>),
    WrappingIntegerArithmetic(JavaIntrinsicMappingInput<WrappingIntegerArithmetic>),
    FloatingPointArithmetic(JavaIntrinsicMappingInput<FloatingPointArithmetic>),
    StringConcatenation(JavaIntrinsicMappingInput<StringConcatenation>),
    IntegerBitwise(JavaIntrinsicMappingInput<IntegerBitwise>),
    CheckedIntegerShifts(JavaIntrinsicMappingInput<CheckedIntegerShifts>),
    FloatingPointInspection(JavaIntrinsicMappingInput<FloatingPointInspection>),
    StringInspection(JavaIntrinsicMappingInput<StringInspection>),
    StringTransformation(JavaIntrinsicMappingInput<StringTransformation>),
    BytesOperations(JavaIntrinsicMappingInput<BytesOperations>),
    ListOperations(JavaIntrinsicMappingInput<ListOperations>),
    OptionOperations(JavaIntrinsicMappingInput<OptionOperations>),
    ResultInspection(JavaIntrinsicMappingInput<ResultInspection>),
    IntegerConversions(JavaIntrinsicMappingInput<IntegerConversions>),
    Utf8Conversions(JavaIntrinsicMappingInput<Utf8Conversions>),
}

pub(crate) fn classify_intrinsic(
    value: CoreIntrinsicExpr<JavaExpr>,
    result: JavaType,
) -> JavaIntrinsicFamily {
    use CoreBinaryIntrinsic as B;
    use CoreUnaryIntrinsic as U;

    enum Family {
        BooleanLogic,
        Equality,
        Ordering,
        CheckedIntegerArithmetic,
        WrappingIntegerArithmetic,
        FloatingPointArithmetic,
        StringConcatenation,
        IntegerBitwise,
        CheckedIntegerShifts,
        FloatingPointInspection,
        StringInspection,
        StringTransformation,
        BytesOperations,
        ListOperations,
        OptionOperations,
        ResultInspection,
        IntegerConversions,
        Utf8Conversions,
    }

    let family = match &value {
        CoreIntrinsicExpr::Unary { operation, .. } => match operation {
            U::BoolNot => Family::BooleanLogic,
            U::IntNegChecked => Family::CheckedIntegerArithmetic,
            U::IntNegWrapping => Family::WrappingIntegerArithmetic,
            U::IntBitNot => Family::IntegerBitwise,
            U::FloatNeg => Family::FloatingPointArithmetic,
            U::FloatTrunc | U::FloatIsNaN | U::FloatIsNegativeZero | U::FloatAbs => {
                Family::FloatingPointInspection
            }
            U::StringScalarLength | U::StringUtf16Length | U::StringIsEmpty => {
                Family::StringInspection
            }
            U::BytesLength | U::BytesIsEmpty => Family::BytesOperations,
            U::ListLength | U::ListIsEmpty => Family::ListOperations,
            U::OptionIsSome | U::OptionIsNone => Family::OptionOperations,
            U::ResultIsOk | U::ResultIsErr => Family::ResultInspection,
            U::WidenI32ToI64 | U::NarrowI64ToI32Checked => Family::IntegerConversions,
            U::StringToUtf8 | U::StringFromUtf8Checked => Family::Utf8Conversions,
        },
        CoreIntrinsicExpr::Binary { operation, .. } => match operation {
            B::BoolAnd | B::BoolOr => Family::BooleanLogic,
            B::Equal | B::NotEqual => Family::Equality,
            B::Less | B::LessEqual | B::Greater | B::GreaterEqual => Family::Ordering,
            B::IntAddChecked
            | B::IntSubChecked
            | B::IntMulChecked
            | B::IntDivChecked
            | B::IntRemChecked => Family::CheckedIntegerArithmetic,
            B::IntAddWrapping | B::IntSubWrapping | B::IntMulWrapping => {
                Family::WrappingIntegerArithmetic
            }
            B::FloatAdd | B::FloatSub | B::FloatMul | B::FloatDiv | B::FloatRemTrunc => {
                Family::FloatingPointArithmetic
            }
            B::StringConcat => Family::StringConcatenation,
            B::IntBitAnd | B::IntBitOr | B::IntBitXor => Family::IntegerBitwise,
            B::IntShiftLeftChecked | B::IntShiftRightChecked => Family::CheckedIntegerShifts,
            B::StringIndexOfLiteral
            | B::StringContains
            | B::StringStartsWith
            | B::StringEndsWith => Family::StringInspection,
            B::StringStripPrefix
            | B::StringTruncateUtf8Bytes
            | B::StringTrimStart
            | B::StringTrimEnd => Family::StringTransformation,
            B::BytesConcat => Family::BytesOperations,
            B::ListGetChecked
            | B::ListAppend
            | B::ListConcat
            | B::ListContains
            | B::ListIndexOf => Family::ListOperations,
            B::OptionUnwrapOr => Family::OptionOperations,
        },
        CoreIntrinsicExpr::Ternary { operation, .. } => match operation {
            CoreTernaryIntrinsic::StringSliceScalars | CoreTernaryIntrinsic::StringReplaceAll => {
                Family::StringTransformation
            }
            CoreTernaryIntrinsic::BytesReplaceAll => Family::BytesOperations,
        },
        CoreIntrinsicExpr::Variadic { operation, .. } => match operation {
            CoreVariadicIntrinsic::StringReplaceMany => Family::StringTransformation,
        },
    };
    match family {
        Family::BooleanLogic => {
            JavaIntrinsicFamily::BooleanLogic(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::Equality => {
            JavaIntrinsicFamily::Equality(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::Ordering => {
            JavaIntrinsicFamily::Ordering(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::CheckedIntegerArithmetic => JavaIntrinsicFamily::CheckedIntegerArithmetic(
            JavaIntrinsicMappingInput::new(value, result),
        ),
        Family::WrappingIntegerArithmetic => JavaIntrinsicFamily::WrappingIntegerArithmetic(
            JavaIntrinsicMappingInput::new(value, result),
        ),
        Family::FloatingPointArithmetic => JavaIntrinsicFamily::FloatingPointArithmetic(
            JavaIntrinsicMappingInput::new(value, result),
        ),
        Family::StringConcatenation => {
            JavaIntrinsicFamily::StringConcatenation(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::IntegerBitwise => {
            JavaIntrinsicFamily::IntegerBitwise(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::CheckedIntegerShifts => {
            JavaIntrinsicFamily::CheckedIntegerShifts(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::FloatingPointInspection => JavaIntrinsicFamily::FloatingPointInspection(
            JavaIntrinsicMappingInput::new(value, result),
        ),
        Family::StringInspection => {
            JavaIntrinsicFamily::StringInspection(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::StringTransformation => {
            JavaIntrinsicFamily::StringTransformation(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::BytesOperations => {
            JavaIntrinsicFamily::BytesOperations(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::ListOperations => {
            JavaIntrinsicFamily::ListOperations(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::OptionOperations => {
            JavaIntrinsicFamily::OptionOperations(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::ResultInspection => {
            JavaIntrinsicFamily::ResultInspection(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::IntegerConversions => {
            JavaIntrinsicFamily::IntegerConversions(JavaIntrinsicMappingInput::new(value, result))
        }
        Family::Utf8Conversions => {
            JavaIntrinsicFamily::Utf8Conversions(JavaIntrinsicMappingInput::new(value, result))
        }
    }
}
