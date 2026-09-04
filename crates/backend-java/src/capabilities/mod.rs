//! Java capability mappings and their exhaustive registration.

use portable_build::{LanguageCapabilityPlugin, implemented_capability_slots};

mod bool_values;
mod boolean_logic;
mod bytes_operations;
mod bytes_values;
mod char_values;
mod checked_integer_arithmetic;
mod checked_integer_shifts;
mod dispatch;
mod equality;
mod f64_values;
mod floating_point_arithmetic;
mod floating_point_inspection;
mod functions;
mod i32_values;
mod i64_values;
mod integer_bitwise;
mod integer_conversions;
mod list_operations;
mod list_values;
mod option_operations;
mod option_values;
mod ordering;
mod records;
mod result_operations;
mod result_values;
mod string_concatenation;
mod string_inspection;
mod string_transformation;
mod support;
mod text_values;
mod utf8_conversions;
mod wrapping_integer_arithmetic;

pub use bool_values::JavaBoolValues;
pub use boolean_logic::JavaBooleanLogic;
pub use bytes_operations::JavaBytesOperations;
pub use bytes_values::JavaBytesValues;
pub use char_values::JavaCharValues;
pub use checked_integer_arithmetic::JavaCheckedIntegerArithmetic;
pub use checked_integer_shifts::JavaCheckedIntegerShifts;
pub use equality::JavaEquality;
pub use f64_values::JavaF64Values;
pub use floating_point_arithmetic::JavaFloatingPointArithmetic;
pub use floating_point_inspection::JavaFloatingPointInspection;
pub use functions::JavaFunctions;
pub use functions::JavaFunctionsNode;
pub use i32_values::JavaI32Values;
pub use i64_values::JavaI64Values;
pub use integer_bitwise::JavaIntegerBitwise;
pub use integer_conversions::JavaIntegerConversions;
pub use list_operations::JavaListOperations;
pub use list_values::JavaListValues;
pub use option_operations::JavaOptionOperations;
pub use option_values::JavaOptionValues;
pub use ordering::JavaOrdering;
pub use records::JavaRecords;
pub use records::JavaRecordsNode;
pub use result_operations::JavaResultOperations;
pub use result_values::JavaResultValues;
pub use string_concatenation::JavaStringConcatenation;
pub use string_inspection::JavaStringInspection;
pub use string_transformation::JavaStringTransformation;
pub use support::{JavaCapabilityMapping, JavaPluginBuilder, java_plugin_builder};
pub use text_values::JavaTextValues;
pub use utf8_conversions::JavaUtf8Conversions;
pub use wrapping_integer_arithmetic::JavaWrappingIntegerArithmetic;

pub(crate) use dispatch::{JavaIntrinsicFamily, classify_intrinsic};

use crate::dialect::JavaDialect;

pub type JavaCapabilitySlots = implemented_capability_slots!(
    JavaFunctions,
    JavaRecords,
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
    JavaIntegerBitwise,
    JavaCheckedIntegerShifts,
    JavaFloatingPointInspection,
    JavaStringInspection,
    JavaStringTransformation,
    JavaBytesOperations,
    JavaListOperations,
    JavaOptionOperations,
    JavaResultOperations,
    JavaIntegerConversions,
    JavaUtf8Conversions,
);

pub type JavaCapabilitySet = LanguageCapabilityPlugin<JavaDialect, JavaCapabilitySlots>;

pub(crate) fn java_capabilities() -> JavaCapabilitySet {
    java_plugin_builder()
        .support(JavaFunctions)
        .support(JavaRecords)
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
        .support(JavaIntegerBitwise)
        .support(JavaCheckedIntegerShifts)
        .support(JavaFloatingPointInspection)
        .support(JavaStringInspection)
        .support(JavaStringTransformation)
        .support(JavaBytesOperations)
        .support(JavaListOperations)
        .support(JavaOptionOperations)
        .support(JavaResultOperations)
        .support(JavaIntegerConversions)
        .support(JavaUtf8Conversions)
        .build()
}
