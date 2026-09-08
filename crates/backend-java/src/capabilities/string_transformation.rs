//! Java mapping for `StringTransformation`.

mod mapping_plan;

use portable_build::StringTransformation;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaKnownType, JavaMemberOrigin, JavaPrimitive, JavaType},
    dialect::{JavaKnownCallable, JavaKnownMethod, JavaRuntimeCallable},
    lower::{JavaIntrinsicExpr, conditional, known_generic_call, member_call, runtime_call},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaStringTransformationInput {
    StripPrefix {
        source: JavaExpr,
        prefix: JavaExpr,
        result: JavaType,
    },
    TruncateUtf8Bytes {
        source: JavaExpr,
        budget: JavaExpr,
        result: JavaType,
    },
    TrimStart {
        source: JavaExpr,
        characters: JavaExpr,
        result: JavaType,
    },
    TrimEnd {
        source: JavaExpr,
        characters: JavaExpr,
        result: JavaType,
    },
    SliceScalars {
        source: JavaExpr,
        start: JavaExpr,
        end: JavaExpr,
        result: JavaType,
    },
    ReplaceAll {
        source: JavaExpr,
        needle: JavaExpr,
        replacement: JavaExpr,
        result: JavaType,
    },
    ReplaceMany {
        source: JavaExpr,
        replacements: Vec<JavaExpr>,
        result: JavaType,
    },
}

fn lower_string_transformation(
    input: JavaStringTransformationInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(JavaIntrinsicExpr::Infallible(match input {
        JavaStringTransformationInput::StripPrefix {
            source,
            prefix,
            result,
        } => {
            let starts = member_call(
                source.clone(),
                "startsWith",
                vec![prefix.clone()],
                JavaType::primitive(JavaPrimitive::Boolean),
                JavaMemberOrigin::Known(JavaKnownMethod::StringStartsWith),
            );
            let length = member_call(
                prefix,
                "length",
                vec![],
                JavaType::primitive(JavaPrimitive::Int),
                JavaMemberOrigin::Known(JavaKnownMethod::StringLength),
            );
            let stripped = member_call(
                source.clone(),
                "substring",
                vec![length],
                result.clone(),
                JavaMemberOrigin::Known(JavaKnownMethod::StringSubstringFrom),
            );
            conditional(starts, stripped, source, result)
        }
        JavaStringTransformationInput::TruncateUtf8Bytes {
            source,
            budget,
            result,
        } => runtime_call(
            JavaRuntimeCallable::StringTruncateUtf8Bytes,
            vec![source, budget],
            result,
        ),
        JavaStringTransformationInput::TrimStart {
            source,
            characters,
            result,
        } => runtime_call(
            JavaRuntimeCallable::StringTrimStart,
            vec![source, characters],
            result,
        ),
        JavaStringTransformationInput::TrimEnd {
            source,
            characters,
            result,
        } => runtime_call(
            JavaRuntimeCallable::StringTrimEnd,
            vec![source, characters],
            result,
        ),
        JavaStringTransformationInput::SliceScalars {
            source,
            start,
            end,
            result,
        } => runtime_call(
            JavaRuntimeCallable::StringSliceScalars,
            vec![source, start, end],
            result,
        ),
        JavaStringTransformationInput::ReplaceAll {
            source,
            needle,
            replacement,
            result,
        } => runtime_call(
            JavaRuntimeCallable::StringReplaceAll,
            vec![source, needle, replacement],
            result,
        ),
        JavaStringTransformationInput::ReplaceMany {
            source,
            replacements,
            result,
        } => {
            let pair_list = JavaType::generic(
                JavaKnownType::List,
                vec![JavaType::known(JavaKnownType::String)],
            );
            runtime_call(
                JavaRuntimeCallable::StringReplaceMany,
                vec![
                    source,
                    known_generic_call(JavaKnownCallable::ListOf, replacements, pair_list),
                ],
                result,
            )
        }
    }))
}

java_operation_mapping!(
    JavaStringTransformation,
    StringTransformation,
    JavaStringTransformationInput,
    JavaIntrinsicExpr,
    lower_string_transformation
);
