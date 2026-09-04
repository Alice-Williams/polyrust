//! Java capability mappings and their exhaustive registration.

use portable_build::{LanguageCapabilityPlugin, capability_slots};

mod bool_values;
mod boolean_logic;
mod bytes_operations;
mod bytes_values;
mod char_values;
mod checked_integer_arithmetic;
mod checked_integer_shifts;
mod conditionals;
mod constants;
mod dispatch;
mod enums;
mod equality;
mod f64_values;
mod floating_point_arithmetic;
mod floating_point_inspection;
mod functions;
mod i32_values;
mod i64_values;
mod integer_bitwise;
mod integer_conversions;
mod interfaces;
mod list_operations;
mod list_values;
mod local_bindings;
mod loops;
mod modules;
mod option_operations;
mod option_values;
mod ordering;
mod pattern_matching;
mod portable_tests;
mod records;
mod result_operations;
mod result_propagation;
mod result_values;
mod string_concatenation;
mod string_inspection;
mod string_transformation;
mod support;
mod text_values;
mod type_aliases;
mod unit_values;
mod utf8_conversions;
mod wrapping_integer_arithmetic;

pub use bool_values::JavaBoolValues;
pub use boolean_logic::JavaBooleanLogic;
pub use bytes_operations::JavaBytesOperations;
pub(crate) use bytes_values::JavaBytesInput;
pub use bytes_values::JavaBytesValues;
pub use char_values::JavaCharValues;
pub use checked_integer_arithmetic::JavaCheckedIntegerArithmetic;
pub use checked_integer_shifts::JavaCheckedIntegerShifts;
pub use conditionals::JavaConditionals;
pub use constants::JavaConstants;
pub use enums::JavaEnums;
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
pub use interfaces::JavaInterfaces;
pub use list_operations::JavaListOperations;
pub(crate) use list_values::JavaListInput;
pub use list_values::JavaListValues;
pub use local_bindings::JavaLocalBindings;
pub use loops::JavaLoops;
pub use modules::JavaModules;
pub use option_operations::JavaOptionOperations;
pub(crate) use option_values::JavaOptionInput;
pub use option_values::JavaOptionValues;
pub use ordering::JavaOrdering;
pub use pattern_matching::JavaPatternMatching;
pub use portable_tests::JavaPortableTests;
pub use records::JavaRecords;
pub use records::JavaRecordsNode;
pub use result_operations::JavaResultOperations;
pub use result_propagation::JavaResultPropagation;
pub(crate) use result_values::JavaResultInput;
pub use result_values::JavaResultValues;
pub use string_concatenation::JavaStringConcatenation;
pub use string_inspection::JavaStringInspection;
pub use string_transformation::JavaStringTransformation;
pub use support::{JavaCapabilityMapping, JavaPluginBuilder, java_plugin_builder};
pub use text_values::JavaTextValues;
pub use type_aliases::JavaTypeAliases;
pub use unit_values::JavaUnitValues;
pub use utf8_conversions::JavaUtf8Conversions;
pub use wrapping_integer_arithmetic::JavaWrappingIntegerArithmetic;

pub(crate) use conditionals::{
    JavaConditionalValueInput, JavaConditionalsInput, JavaConditionalsNode,
};
pub(crate) use constants::{JavaConstantsInput, JavaConstantsNode};
pub(crate) use dispatch::{JavaIntrinsicFamily, classify_intrinsic};
pub(crate) use enums::{JavaEnumBranchInput, JavaEnumVariantInput, JavaEnumsInput, JavaEnumsNode};
pub(crate) use functions::{JavaFunctionDeclarationInput, JavaFunctionsInput};
pub(crate) use interfaces::{
    JavaConcreteInterfaceCallInput, JavaInterfaceCallInput, JavaInterfaceDeclarationInput,
    JavaInterfaceImplementationInput, JavaInterfaceMethodInput, JavaInterfacesInput,
    JavaInterfacesNode,
};
pub(crate) use local_bindings::JavaLocalBindingInput;
pub(crate) use loops::JavaLoopsInput;
pub(crate) use modules::JavaModuleInput;
pub(crate) use pattern_matching::{
    JavaLoweredPattern, JavaMatchArmInput, JavaMatchInput, JavaPatternFieldBindingInput,
    JavaPatternInput, JavaPatternMatchPlan, JavaPatternMatchingInput, JavaPatternMatchingNode,
};
pub(crate) use portable_tests::{
    JavaPortableTestCaseInput, JavaPortableTestExpectation, JavaPortableTestHarnessInput,
    JavaPortableTestsInput, JavaPortableTestsNode,
};
pub(crate) use records::{JavaRecordDeclarationInput, JavaRecordsInput};
pub(crate) use result_propagation::{JavaResultPropagationInput, JavaResultPropagationPlan};
pub(crate) use type_aliases::JavaTypeAliasInput;

use crate::dialect::JavaDialect;
use support::{ObservedJavaMapping, observed};

#[cfg(test)]
pub(crate) use support::{java_mapping_invocations, reset_java_mapping_invocations};

pub type JavaCapabilitySlots = capability_slots!(
    implemented ObservedJavaMapping<JavaFunctions>,
    implemented ObservedJavaMapping<JavaRecords>,
    implemented ObservedJavaMapping<JavaBoolValues>,
    implemented ObservedJavaMapping<JavaI32Values>,
    implemented ObservedJavaMapping<JavaI64Values>,
    implemented ObservedJavaMapping<JavaF64Values>,
    implemented ObservedJavaMapping<JavaTextValues>,
    implemented ObservedJavaMapping<JavaBooleanLogic>,
    implemented ObservedJavaMapping<JavaEquality>,
    implemented ObservedJavaMapping<JavaOrdering>,
    implemented ObservedJavaMapping<JavaCheckedIntegerArithmetic>,
    implemented ObservedJavaMapping<JavaWrappingIntegerArithmetic>,
    implemented ObservedJavaMapping<JavaFloatingPointArithmetic>,
    implemented ObservedJavaMapping<JavaStringConcatenation>,
    implemented ObservedJavaMapping<JavaCharValues>,
    implemented ObservedJavaMapping<JavaBytesValues>,
    implemented ObservedJavaMapping<JavaListValues>,
    implemented ObservedJavaMapping<JavaOptionValues>,
    implemented ObservedJavaMapping<JavaResultValues>,
    implemented ObservedJavaMapping<JavaIntegerBitwise>,
    implemented ObservedJavaMapping<JavaCheckedIntegerShifts>,
    implemented ObservedJavaMapping<JavaFloatingPointInspection>,
    implemented ObservedJavaMapping<JavaStringInspection>,
    implemented ObservedJavaMapping<JavaStringTransformation>,
    implemented ObservedJavaMapping<JavaBytesOperations>,
    implemented ObservedJavaMapping<JavaListOperations>,
    implemented ObservedJavaMapping<JavaOptionOperations>,
    implemented ObservedJavaMapping<JavaResultOperations>,
    implemented ObservedJavaMapping<JavaIntegerConversions>,
    implemented ObservedJavaMapping<JavaUtf8Conversions>,
    implemented ObservedJavaMapping<JavaModules>,
    implemented ObservedJavaMapping<JavaConstants>,
    implemented ObservedJavaMapping<JavaTypeAliases>,
    implemented ObservedJavaMapping<JavaEnums>,
    implemented ObservedJavaMapping<JavaInterfaces>,
    implemented ObservedJavaMapping<JavaPortableTests>,
    implemented ObservedJavaMapping<JavaLocalBindings>,
    implemented ObservedJavaMapping<JavaConditionals>,
    implemented ObservedJavaMapping<JavaLoops>,
    implemented ObservedJavaMapping<JavaPatternMatching>,
    implemented ObservedJavaMapping<JavaResultPropagation>,
    implemented ObservedJavaMapping<JavaUnitValues>,
);

pub type JavaCapabilitySet = LanguageCapabilityPlugin<JavaDialect, JavaCapabilitySlots>;

pub(crate) fn java_capabilities() -> JavaCapabilitySet {
    java_plugin_builder()
        .support(observed(JavaFunctions))
        .support(observed(JavaRecords))
        .support(observed(JavaBoolValues))
        .support(observed(JavaI32Values))
        .support(observed(JavaI64Values))
        .support(observed(JavaF64Values))
        .support(observed(JavaTextValues))
        .support(observed(JavaBooleanLogic))
        .support(observed(JavaEquality))
        .support(observed(JavaOrdering))
        .support(observed(JavaCheckedIntegerArithmetic))
        .support(observed(JavaWrappingIntegerArithmetic))
        .support(observed(JavaFloatingPointArithmetic))
        .support(observed(JavaStringConcatenation))
        .support(observed(JavaCharValues))
        .support(observed(JavaBytesValues))
        .support(observed(JavaListValues))
        .support(observed(JavaOptionValues))
        .support(observed(JavaResultValues))
        .support(observed(JavaIntegerBitwise))
        .support(observed(JavaCheckedIntegerShifts))
        .support(observed(JavaFloatingPointInspection))
        .support(observed(JavaStringInspection))
        .support(observed(JavaStringTransformation))
        .support(observed(JavaBytesOperations))
        .support(observed(JavaListOperations))
        .support(observed(JavaOptionOperations))
        .support(observed(JavaResultOperations))
        .support(observed(JavaIntegerConversions))
        .support(observed(JavaUtf8Conversions))
        .support(observed(JavaModules))
        .support(observed(JavaConstants))
        .support(observed(JavaTypeAliases))
        .support(observed(JavaEnums))
        .support(observed(JavaInterfaces))
        .support(observed(JavaPortableTests))
        .support(observed(JavaLocalBindings))
        .support(observed(JavaConditionals))
        .support(observed(JavaLoops))
        .support(observed(JavaPatternMatching))
        .support(observed(JavaResultPropagation))
        .support(observed(JavaUnitValues))
        .build()
}
