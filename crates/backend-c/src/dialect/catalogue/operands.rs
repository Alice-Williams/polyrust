//! Typed binding of obligations to actual call operands, not safety evidence.

use super::CKnownCall;
use crate::ast::CValue;

/// The verifier must discharge the documented obligation on these exact values
/// before applying a transition. Constructing this borrowed view proves no
/// range, ownership, initialization, allocator or resource property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CKnownOperands<'a> {
    /// Positive checked bytes; null or fresh default-allocator storage.
    Allocate {
        bytes: &'a CValue,
    },
    /// Null or the exact live default-allocator base, with no outstanding borrow.
    Release {
        base: &'a CValue,
    },
    /// Checked nonoverlapping writable/readable extents; source initialized.
    /// The result aliases destination and does not create a new owner.
    CopyBytes {
        destination: &'a CValue,
        source: &'a CValue,
        bytes: &'a CValue,
    },
    /// Initialized byte spans, not structural equality over aggregate padding.
    CompareBytes {
        left: &'a CValue,
        right: &'a CValue,
        bytes: &'a CValue,
    },
    /// Platform floating environment; errno/sticky flags are not preserved.
    FloatRemainder {
        dividend: &'a CValue,
        divisor: &'a CValue,
    },
    FloatTruncate {
        value: &'a CValue,
    },
    /// Int zero/nonzero, not a normalized Boolean representation.
    IsNan {
        value: &'a CValue,
    },
    SignBit {
        value: &'a CValue,
    },
    /// Checked product, initialized extent and valid borrowed stream. The result
    /// counts complete elements; output is not transactional and may be partial.
    WriteBytes {
        source: &'a CValue,
        element_size: &'a CValue,
        count: &'a CValue,
        stream: &'a CValue,
    },
    /// Valid borrowed stream; Int zero/nonzero, without FILE ownership.
    StreamError {
        stream: &'a CValue,
    },
}

impl CKnownCall {
    pub(crate) fn bind(self, arguments: &[CValue]) -> CKnownOperands<'_> {
        // Only CCall's constructor-checked argument list reaches this projection.
        // Contextual verification reconstructs private mutations before analysis.
        match (self, arguments) {
            (Self::Allocate, [bytes]) => CKnownOperands::Allocate { bytes },
            (Self::Release, [base]) => CKnownOperands::Release { base },
            (Self::CopyBytes, [destination, source, bytes]) => CKnownOperands::CopyBytes {
                destination,
                source,
                bytes,
            },
            (Self::CompareBytes, [left, right, bytes]) => {
                CKnownOperands::CompareBytes { left, right, bytes }
            }
            (Self::FloatRemainder, [dividend, divisor]) => {
                CKnownOperands::FloatRemainder { dividend, divisor }
            }
            (Self::FloatTruncate, [value]) => CKnownOperands::FloatTruncate { value },
            (Self::IsNan, [value]) => CKnownOperands::IsNan { value },
            (Self::SignBit, [value]) => CKnownOperands::SignBit { value },
            (Self::WriteBytes, [source, element_size, count, stream]) => {
                CKnownOperands::WriteBytes {
                    source,
                    element_size,
                    count,
                    stream,
                }
            }
            (Self::StreamError, [stream]) => CKnownOperands::StreamError { stream },
            _ => unreachable!("constructor-checked known call arity"),
        }
    }
}
