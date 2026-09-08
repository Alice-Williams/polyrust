use super::*;
#[test]
fn string_concatenation_all_variants() {
    verify(
        c::JavaStringConcatenation,
        c::string_concatenation::JavaStringConcatenationInput {
            left: text(),
            right: text(),
            result: string(),
        },
        R::Direct,
    );
}
#[test]
fn string_inspection_all_variants() {
    verify(
        c::JavaStringInspection,
        c::string_inspection::JavaStringInspectionInput::ScalarLength {
            source: text(),
            result: long(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaStringInspection,
        c::string_inspection::JavaStringInspectionInput::Utf16Length {
            source: text(),
            result: long(),
        },
        R::Direct,
    );
    verify(
        c::JavaStringInspection,
        c::string_inspection::JavaStringInspectionInput::IsEmpty {
            source: text(),
            result: boolean(),
        },
        R::Direct,
    );
    verify(
        c::JavaStringInspection,
        c::string_inspection::JavaStringInspectionInput::IndexOfLiteral {
            source: text(),
            needle: text(),
            result: long(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaStringInspection,
        c::string_inspection::JavaStringInspectionInput::Contains {
            source: text(),
            needle: text(),
            result: boolean(),
        },
        R::Direct,
    );
    verify(
        c::JavaStringInspection,
        c::string_inspection::JavaStringInspectionInput::StartsWith {
            source: text(),
            prefix: text(),
            result: boolean(),
        },
        R::Direct,
    );
    verify(
        c::JavaStringInspection,
        c::string_inspection::JavaStringInspectionInput::EndsWith {
            source: text(),
            suffix: text(),
            result: boolean(),
        },
        R::Direct,
    );
}
#[test]
fn string_transformation_all_variants() {
    verify(
        c::JavaStringTransformation,
        c::string_transformation::JavaStringTransformationInput::StripPrefix {
            source: text(),
            prefix: text(),
            result: string(),
        },
        R::Direct,
    );
    verify(
        c::JavaStringTransformation,
        c::string_transformation::JavaStringTransformationInput::TruncateUtf8Bytes {
            source: text(),
            budget: wide(),
            result: string(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaStringTransformation,
        c::string_transformation::JavaStringTransformationInput::TrimStart {
            source: text(),
            characters: text(),
            result: string(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaStringTransformation,
        c::string_transformation::JavaStringTransformationInput::TrimEnd {
            source: text(),
            characters: text(),
            result: string(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaStringTransformation,
        c::string_transformation::JavaStringTransformationInput::SliceScalars {
            source: text(),
            start: wide(),
            end: wide(),
            result: string(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaStringTransformation,
        c::string_transformation::JavaStringTransformationInput::ReplaceAll {
            source: text(),
            needle: text(),
            replacement: text(),
            result: string(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaStringTransformation,
        c::string_transformation::JavaStringTransformationInput::ReplaceMany {
            source: text(),
            replacements: vec![text(), text()],
            result: string(),
        },
        R::RuntimeHelper,
    );
}
#[test]
fn utf8_conversions_all_variants() {
    verify(
        c::JavaUtf8Conversions,
        c::utf8_conversions::JavaUtf8ConversionsInput::Encode {
            operand: text(),
            result: bytes(),
        },
        R::RuntimeHelper,
    );
    verify(
        c::JavaUtf8Conversions,
        c::utf8_conversions::JavaUtf8ConversionsInput::DecodeChecked {
            operand: value(bytes()),
            result: string(),
        },
        R::RuntimeHelper,
    );
}
