//! Java dialect: signature builder.

use crate::ast::{JavaMethodSignature, JavaType};

pub(super) fn signature(
    receiver: Option<JavaType>,
    parameters: Vec<JavaType>,
    result: JavaType,
) -> JavaMethodSignature {
    JavaMethodSignature {
        receiver,
        parameters,
        result,
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    }
}
