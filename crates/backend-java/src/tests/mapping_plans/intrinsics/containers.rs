use super::*;
#[test]
fn bytes_operations_all_variants() {
    verify(
        c::JavaBytesOperations,
        c::bytes_operations::JavaBytesOperationsInput::Length {
            bytes: value(bytes()),
            result: long(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaBytesOperations,
        c::bytes_operations::JavaBytesOperationsInput::IsEmpty {
            bytes: value(bytes()),
            result: boolean(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaBytesOperations,
        c::bytes_operations::JavaBytesOperationsInput::Concat {
            left: value(bytes()),
            right: value(bytes()),
            result: bytes(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaBytesOperations,
        c::bytes_operations::JavaBytesOperationsInput::ReplaceAll {
            source: value(bytes()),
            needle: value(bytes()),
            replacement: Box::new(value(bytes())),
            result: bytes(),
        },
        R::RuntimeHelper,
    );
}
#[test]
fn list_operations_all_variants() {
    verify(
        c::JavaListOperations,
        c::list_operations::JavaListOperationsInput::Length {
            list: value(list()),
            result: long(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaListOperations,
        c::list_operations::JavaListOperationsInput::IsEmpty {
            list: value(list()),
            result: boolean(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaListOperations,
        c::list_operations::JavaListOperationsInput::GetChecked {
            list: value(list()),
            index: integer(),
            result: JavaType::primitive(JavaPrimitive::Int),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaListOperations,
        c::list_operations::JavaListOperationsInput::Append {
            list: value(list()),
            value: integer(),
            result: list(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaListOperations,
        c::list_operations::JavaListOperationsInput::Concat {
            left: value(list()),
            right: value(list()),
            result: list(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaListOperations,
        c::list_operations::JavaListOperationsInput::Contains {
            list: value(list()),
            value: integer(),
            result: boolean(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaListOperations,
        c::list_operations::JavaListOperationsInput::IndexOf {
            list: value(list()),
            value: integer(),
            result: long(),
        },
        R::RuntimeHelper,
    );
}
#[test]
fn option_operations_all_variants() {
    verify(
        c::JavaOptionOperations,
        c::option_operations::JavaOptionOperationsInput::IsSome {
            operand: value(option()),
            result: boolean(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaOptionOperations,
        c::option_operations::JavaOptionOperationsInput::IsNone {
            operand: value(option()),
            result: boolean(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaOptionOperations,
        c::option_operations::JavaOptionOperationsInput::UnwrapOr {
            option: value(option()),
            fallback: Box::new(integer()),
            result: JavaType::primitive(JavaPrimitive::Int),
        },
        R::RuntimeHelper,
    );
}
#[test]
fn result_operations_all_variants() {
    verify(
        c::JavaResultOperations,
        c::result_operations::JavaResultOperationsInput::IsOk {
            operand: value(result()),
            result: boolean(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaResultOperations,
        c::result_operations::JavaResultOperationsInput::IsErr {
            operand: value(result()),
            result: boolean(),
        },
        R::RuntimeHelper,
    );
}
