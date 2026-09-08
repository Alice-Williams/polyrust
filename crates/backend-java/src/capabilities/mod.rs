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
pub use support::{
    JavaCapabilityMapping, JavaMappingOutput, JavaPluginBuilder, java_plugin_builder,
};
pub use text_values::JavaTextValues;
pub use type_aliases::JavaTypeAliases;
pub use unit_values::JavaUnitValues;
pub use utf8_conversions::JavaUtf8Conversions;
pub use wrapping_integer_arithmetic::JavaWrappingIntegerArithmetic;

pub(crate) use bool_values::JavaBoolValuesInput;
pub(crate) use boolean_logic::{JavaBooleanLogicInput, JavaBooleanLogicPlan};
pub(crate) use bytes_operations::JavaBytesOperationsInput;
pub(crate) use char_values::JavaCharValuesInput;
pub(crate) use checked_integer_arithmetic::JavaCheckedIntegerArithmeticInput;
pub(crate) use checked_integer_shifts::JavaCheckedIntegerShiftsInput;
pub(crate) use conditionals::{
    JavaConditionalValueInput, JavaConditionalsInput, JavaConditionalsNode,
};
pub(crate) use constants::{JavaConstantsInput, JavaConstantsNode};
pub(crate) use dispatch::{JavaIntrinsicFamily, classify_intrinsic};
pub(crate) use enums::{
    JavaEnumBranchInput, JavaEnumEqualityOperator, JavaEnumPayloadVariantInput, JavaEnumShape,
    JavaEnumVariantInput, JavaEnumsInput, JavaEnumsNode,
};
pub(crate) use equality::JavaEqualityInput;
pub(crate) use f64_values::JavaF64ValuesInput;
pub(crate) use floating_point_arithmetic::JavaFloatingPointArithmeticInput;
pub(crate) use floating_point_inspection::JavaFloatingPointInspectionInput;
pub(crate) use functions::{JavaFunctionDeclarationInput, JavaFunctionsInput};
pub(crate) use i32_values::JavaI32ValuesInput;
pub(crate) use i64_values::JavaI64ValuesInput;
pub(crate) use integer_bitwise::JavaIntegerBitwiseInput;
pub(crate) use integer_conversions::JavaIntegerConversionsInput;
pub(crate) use interfaces::{
    JavaConcreteInterfaceCallInput, JavaInterfaceCallInput, JavaInterfaceConformanceInput,
    JavaInterfaceConformancePlan, JavaInterfaceDeclarationInput, JavaInterfaceImplementationInput,
    JavaInterfaceMethodInput, JavaInterfacesInput, JavaInterfacesNode,
    JavaUninhabitedInterfaceInput,
};
pub(crate) use list_operations::JavaListOperationsInput;
pub(crate) use local_bindings::{JavaLocalBindingsInput, JavaLocalBindingsNode};
pub(crate) use loops::{JavaLoopsInput, JavaLoopsNode};
pub(crate) use modules::JavaModuleInput;
pub(crate) use option_operations::JavaOptionOperationsInput;
pub(crate) use ordering::JavaOrderingInput;
pub(crate) use pattern_matching::{
    JavaLoweredPattern, JavaMatchArmInput, JavaMatchInput, JavaPatternFieldBindingInput,
    JavaPatternInput, JavaPatternMatchPlan, JavaPatternMatchingInput, JavaPatternMatchingNode,
};
pub(crate) use portable_tests::{
    JavaPortableFunctionInvocationInput, JavaPortableMethodInvocationInput,
    JavaPortableTestCaseInput, JavaPortableTestExpectation, JavaPortableTestHarnessInput,
    JavaPortableTestsInput, JavaPortableTestsNode,
};
pub(crate) use records::{JavaRecordDeclarationInput, JavaRecordsInput};
pub(crate) use result_operations::JavaResultOperationsInput;
pub(crate) use result_propagation::{JavaResultPropagationInput, JavaResultPropagationPlan};
pub(crate) use string_concatenation::JavaStringConcatenationInput;
pub(crate) use string_inspection::JavaStringInspectionInput;
pub(crate) use string_transformation::JavaStringTransformationInput;
pub(crate) use support::JavaValueNode;
pub(crate) use text_values::JavaTextValuesInput;
pub(crate) use type_aliases::JavaTypeAliasInput;
pub(crate) use unit_values::JavaUnitValuesInput;
pub(crate) use utf8_conversions::JavaUtf8ConversionsInput;
pub(crate) use wrapping_integer_arithmetic::JavaWrappingIntegerArithmeticInput;

use crate::dialect::JavaDialect;
use support::CheckedJavaMapping;

#[cfg(test)]
#[path = "../tests/mapping_plans/mod.rs"]
mod mapping_plan_tests;

#[cfg(test)]
pub(crate) use support::{
    java_mapping_invocations, java_mapping_operation_counts, reset_java_mapping_invocations,
};

pub type JavaCapabilitySlots = capability_slots!(
    implemented CheckedJavaMapping<JavaFunctions>,
    implemented CheckedJavaMapping<JavaRecords>,
    implemented CheckedJavaMapping<JavaBoolValues>,
    implemented CheckedJavaMapping<JavaI32Values>,
    implemented CheckedJavaMapping<JavaI64Values>,
    implemented CheckedJavaMapping<JavaF64Values>,
    implemented CheckedJavaMapping<JavaTextValues>,
    implemented CheckedJavaMapping<JavaBooleanLogic>,
    implemented CheckedJavaMapping<JavaEquality>,
    implemented CheckedJavaMapping<JavaOrdering>,
    implemented CheckedJavaMapping<JavaCheckedIntegerArithmetic>,
    implemented CheckedJavaMapping<JavaWrappingIntegerArithmetic>,
    implemented CheckedJavaMapping<JavaFloatingPointArithmetic>,
    implemented CheckedJavaMapping<JavaStringConcatenation>,
    implemented CheckedJavaMapping<JavaCharValues>,
    implemented CheckedJavaMapping<JavaBytesValues>,
    implemented CheckedJavaMapping<JavaListValues>,
    implemented CheckedJavaMapping<JavaOptionValues>,
    implemented CheckedJavaMapping<JavaResultValues>,
    implemented CheckedJavaMapping<JavaIntegerBitwise>,
    implemented CheckedJavaMapping<JavaCheckedIntegerShifts>,
    implemented CheckedJavaMapping<JavaFloatingPointInspection>,
    implemented CheckedJavaMapping<JavaStringInspection>,
    implemented CheckedJavaMapping<JavaStringTransformation>,
    implemented CheckedJavaMapping<JavaBytesOperations>,
    implemented CheckedJavaMapping<JavaListOperations>,
    implemented CheckedJavaMapping<JavaOptionOperations>,
    implemented CheckedJavaMapping<JavaResultOperations>,
    implemented CheckedJavaMapping<JavaIntegerConversions>,
    implemented CheckedJavaMapping<JavaUtf8Conversions>,
    implemented CheckedJavaMapping<JavaModules>,
    implemented CheckedJavaMapping<JavaConstants>,
    implemented CheckedJavaMapping<JavaTypeAliases>,
    implemented CheckedJavaMapping<JavaEnums>,
    implemented CheckedJavaMapping<JavaInterfaces>,
    implemented CheckedJavaMapping<JavaPortableTests>,
    implemented CheckedJavaMapping<JavaLocalBindings>,
    implemented CheckedJavaMapping<JavaConditionals>,
    implemented CheckedJavaMapping<JavaLoops>,
    implemented CheckedJavaMapping<JavaPatternMatching>,
    implemented CheckedJavaMapping<JavaResultPropagation>,
    implemented CheckedJavaMapping<JavaUnitValues>,
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
        .support(JavaModules)
        .support(JavaConstants)
        .support(JavaTypeAliases)
        .support(JavaEnums)
        .support(JavaInterfaces)
        .support(JavaPortableTests)
        .support(JavaLocalBindings)
        .support(JavaConditionals)
        .support(JavaLoops)
        .support(JavaPatternMatching)
        .support(JavaResultPropagation)
        .support(JavaUnitValues)
        .build()
}
