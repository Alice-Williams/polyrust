//! Class-file descriptor/signature lengths, not rendered Java source lengths.

use super::{Names, limit};
use crate::ast::{JavaIdentifier, JavaKnownType, JavaPrimitive, JavaType, JavaTypeName};
use portable_diagnostics::Diagnostic;

pub(super) const MAX_UTF8: usize = 65_535;

#[derive(Clone, Copy)]
pub(super) struct Encoding {
    pub descriptor: usize,
    pub signature: usize,
}

pub(super) fn slots(ty: &JavaType) -> usize {
    match ty {
        JavaType::Primitive(JavaPrimitive::Long | JavaPrimitive::Double) => 2,
        JavaType::Primitive(_)
        | JavaType::Boxed(_)
        | JavaType::Reference(_)
        | JavaType::Array { .. }
        | JavaType::Generic { .. }
        | JavaType::Wildcard { .. }
        | JavaType::TypeVariable(_) => 1,
    }
}

pub(super) fn parameters(
    values: &[JavaIdentifier],
    path: &str,
    errors: &mut Vec<Diagnostic>,
) -> usize {
    if values.is_empty() {
        return 0;
    }
    let size = values.iter().fold(2usize, |size, name| {
        limit(
            errors,
            path,
            "type parameter name bytes",
            name.as_str().len(),
            MAX_UTF8,
        );
        size.saturating_add(name.as_str().len())
            .saturating_add(":Ljava/lang/Object;".len())
    });
    limit(
        errors,
        path,
        "type parameter signature bytes",
        size,
        MAX_UTF8,
    );
    size
}

pub(super) fn encoding(
    ty: &JavaType,
    names: &Names,
    path: &str,
    errors: &mut Vec<Diagnostic>,
) -> Encoding {
    let result = match ty {
        JavaType::Primitive(_) => Encoding {
            descriptor: 1,
            signature: 1,
        },
        JavaType::Boxed(primitive) => reference(
            &JavaTypeName::Known(match primitive {
                JavaPrimitive::Boolean => JavaKnownType::Boolean,
                JavaPrimitive::Byte => JavaKnownType::Byte,
                JavaPrimitive::Char => JavaKnownType::Character,
                JavaPrimitive::Int => JavaKnownType::Integer,
                JavaPrimitive::Long => JavaKnownType::Long,
                JavaPrimitive::Double => JavaKnownType::Double,
                JavaPrimitive::Void => JavaKnownType::Object,
            }),
            names,
        ),
        JavaType::Reference(name) => reference(name, names),
        JavaType::Array { .. } => {
            let mut dimensions = 0usize;
            let mut component = ty;
            while let JavaType::Array {
                component: next, ..
            } = component
            {
                dimensions = dimensions.saturating_add(1);
                component = next;
            }
            limit(errors, path, "array dimensions", dimensions, 255);
            let element = encoding(component, names, path, errors);
            Encoding {
                descriptor: dimensions.saturating_add(element.descriptor),
                signature: dimensions.saturating_add(element.signature),
            }
        }
        JavaType::Generic { raw, arguments } => {
            let base = reference(raw, names);
            Encoding {
                descriptor: base.descriptor,
                signature: arguments.iter().fold(
                    base.signature.saturating_add(2),
                    |size, value| {
                        size.saturating_add(encoding(value, names, path, errors).signature)
                    },
                ),
            }
        }
        JavaType::TypeVariable(name) => {
            limit(
                errors,
                path,
                "type variable name bytes",
                name.as_str().len(),
                MAX_UTF8,
            );
            Encoding {
                descriptor: "Ljava/lang/Object;".len(),
                signature: name.as_str().len().saturating_add(2),
            }
        }
        JavaType::Wildcard { bound } => Encoding {
            descriptor: "Ljava/lang/Object;".len(),
            signature: match bound {
                None => 1,
                Some((_, ty)) => encoding(ty, names, path, errors)
                    .signature
                    .saturating_add(1),
            },
        },
    };
    limit(
        errors,
        path,
        "type descriptor bytes",
        result.descriptor,
        MAX_UTF8,
    );
    limit(
        errors,
        path,
        "generic type signature bytes",
        result.signature,
        MAX_UTF8,
    );
    result
}

fn reference(name: &JavaTypeName, names: &Names) -> Encoding {
    // Dots and binary-name '$' separators each occupy one byte. Identifiers
    // are ASCII by construction; missing identities are rejected by linking.
    let size = match name {
        JavaTypeName::Known(value) => value.qualified_name().len(),
        JavaTypeName::Generated(id) => names.get(id).copied().unwrap_or(0),
    }
    .saturating_add(2);
    Encoding {
        descriptor: size,
        signature: size,
    }
}
