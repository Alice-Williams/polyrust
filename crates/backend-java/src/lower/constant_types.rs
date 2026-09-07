//! Java lowering: constant types.

use super::{Lowering, diagnostic, source};
use crate::ast::{JavaExpr, JavaType};
use portable_core_ir::{
    CoreBinaryIntrinsic, CoreConstantExpr, CoreConstantExprKind, CoreIntrinsicExpr,
    CoreTernaryIntrinsic, CoreType, CoreTypeId, CoreUnaryIntrinsic, CoreValue,
    CoreVariadicIntrinsic,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode};

impl Lowering<'_> {
    pub(super) fn constant_untyped(
        &self,
        value: &CoreConstantExpr,
    ) -> Result<(JavaExpr, JavaType), Vec<Diagnostic>> {
        let core_type = self.constant_type(value)?;
        let java_type = self.ty(core_type)?;
        Ok((self.constant_expr(value, core_type)?, java_type))
    }

    fn constant_type(&self, value: &CoreConstantExpr) -> Result<CoreTypeId, Vec<Diagnostic>> {
        match &value.kind {
            CoreConstantExprKind::Literal(value) => self.constant_value_type(value),
            CoreConstantExprKind::Constant(id) => Ok(self
                .core
                .constant(*id)
                .expect("verified constant reference")
                .ty),
            CoreConstantExprKind::Record { record, .. } => {
                self.find_type(&CoreType::Record(*record))
            }
            CoreConstantExprKind::Enum { enumeration, .. } => {
                self.find_type(&CoreType::Enum(*enumeration))
            }
            CoreConstantExprKind::Some(inner) => {
                let inner = self.constant_type(inner)?;
                self.find_type(&CoreType::Option(inner))
            }
            CoreConstantExprKind::None { inner } => self.find_type(&CoreType::Option(*inner)),
            CoreConstantExprKind::Ok { value, error } => {
                let ok = self.constant_type(value)?;
                self.find_type(&CoreType::Result { ok, error: *error })
            }
            CoreConstantExprKind::Err { value, ok } => {
                let error = self.constant_type(value)?;
                self.find_type(&CoreType::Result { ok: *ok, error })
            }
            CoreConstantExprKind::List { element, .. } => self.find_type(&CoreType::List(*element)),
            CoreConstantExprKind::Intrinsic(intrinsic) => self.constant_intrinsic_type(intrinsic),
        }
    }

    fn constant_value_type(&self, value: &CoreValue) -> Result<CoreTypeId, Vec<Diagnostic>> {
        match value {
            CoreValue::Unit => self.find_type(&CoreType::Unit),
            CoreValue::Bool(_) => self.find_type(&CoreType::Bool),
            CoreValue::I32(_) => self.find_type(&CoreType::I32),
            CoreValue::I64(_) => self.find_type(&CoreType::I64),
            CoreValue::F64(_) => self.find_type(&CoreType::F64),
            CoreValue::Char(_) => self.find_type(&CoreType::Char),
            CoreValue::String(_) => self.find_type(&CoreType::String),
            CoreValue::Bytes(_) => self.find_type(&CoreType::Bytes),
            CoreValue::List(values) if !values.is_empty() => {
                let element = self.constant_value_type(&values[0])?;
                self.find_type(&CoreType::List(element))
            }
            CoreValue::Some(value) => {
                let inner = self.constant_value_type(value)?;
                self.find_type(&CoreType::Option(inner))
            }
            CoreValue::Record { record, .. } => self.find_type(&CoreType::Record(*record)),
            CoreValue::Enum { enumeration, .. } => self.find_type(&CoreType::Enum(*enumeration)),
            CoreValue::None | CoreValue::List(_) | CoreValue::Ok(_) | CoreValue::Err(_) => {
                Err(vec![Diagnostic::error(
                    DiagnosticCode::InvalidStructure,
                    "verified nested constant value has no inferable declared Core type",
                    source("constant-type"),
                )])
            }
        }
    }

    fn constant_intrinsic_type(
        &self,
        value: &CoreIntrinsicExpr<CoreConstantExpr>,
    ) -> Result<CoreTypeId, Vec<Diagnostic>> {
        use CoreBinaryIntrinsic as B;
        use CoreUnaryIntrinsic as U;

        let result = match value {
            CoreIntrinsicExpr::Unary { operation, operand } => {
                let operand = self.constant_type(operand)?;
                match operation {
                    U::BoolNot
                    | U::FloatIsNaN
                    | U::FloatIsNegativeZero
                    | U::StringIsEmpty
                    | U::BytesIsEmpty
                    | U::ListIsEmpty
                    | U::OptionIsSome
                    | U::OptionIsNone
                    | U::ResultIsOk
                    | U::ResultIsErr => CoreType::Bool,
                    U::IntNegChecked | U::IntNegWrapping | U::IntBitNot => {
                        return Ok(operand);
                    }
                    U::FloatNeg | U::FloatTrunc | U::FloatAbs => CoreType::F64,
                    U::StringScalarLength
                    | U::StringUtf16Length
                    | U::BytesLength
                    | U::ListLength
                    | U::WidenI32ToI64 => CoreType::I64,
                    U::NarrowI64ToI32Checked => CoreType::I32,
                    U::StringToUtf8 => CoreType::Bytes,
                    U::StringFromUtf8Checked => CoreType::String,
                }
            }
            CoreIntrinsicExpr::Binary {
                operation,
                left,
                right,
            } => {
                let left = self.constant_type(left)?;
                let right = self.constant_type(right)?;
                match operation {
                    B::BoolAnd
                    | B::BoolOr
                    | B::Equal
                    | B::NotEqual
                    | B::Less
                    | B::LessEqual
                    | B::Greater
                    | B::GreaterEqual
                    | B::StringContains
                    | B::StringStartsWith
                    | B::StringEndsWith
                    | B::ListContains => CoreType::Bool,
                    B::IntAddChecked
                    | B::IntSubChecked
                    | B::IntMulChecked
                    | B::IntDivChecked
                    | B::IntRemChecked
                    | B::IntAddWrapping
                    | B::IntSubWrapping
                    | B::IntMulWrapping
                    | B::IntBitAnd
                    | B::IntBitOr
                    | B::IntBitXor
                    | B::IntShiftLeftChecked
                    | B::IntShiftRightChecked
                    | B::ListAppend
                    | B::ListConcat => return Ok(left),
                    B::FloatAdd | B::FloatSub | B::FloatMul | B::FloatDiv | B::FloatRemTrunc => {
                        CoreType::F64
                    }
                    B::StringConcat
                    | B::StringStripPrefix
                    | B::StringTruncateUtf8Bytes
                    | B::StringTrimStart
                    | B::StringTrimEnd => CoreType::String,
                    B::BytesConcat => CoreType::Bytes,
                    B::StringIndexOfLiteral | B::ListIndexOf => {
                        let i64_type = self.find_type(&CoreType::I64)?;
                        return self.find_type(&CoreType::Option(i64_type));
                    }
                    B::ListGetChecked => match self.core.types().get(left) {
                        Some(CoreType::List(inner)) => return Ok(*inner),
                        _ => {
                            return Err(vec![diagnostic(
                                "verified list-get constant operand is not a list",
                            )]);
                        }
                    },
                    B::OptionUnwrapOr => return Ok(right),
                }
            }
            CoreIntrinsicExpr::Ternary { operation, .. } => match operation {
                CoreTernaryIntrinsic::StringSliceScalars
                | CoreTernaryIntrinsic::StringReplaceAll => CoreType::String,
                CoreTernaryIntrinsic::BytesReplaceAll => CoreType::Bytes,
            },
            CoreIntrinsicExpr::Variadic { operation, .. } => match operation {
                CoreVariadicIntrinsic::StringReplaceMany => CoreType::String,
            },
        };
        self.find_type(&result)
    }

    fn find_type(&self, wanted: &CoreType) -> Result<CoreTypeId, Vec<Diagnostic>> {
        self.core
            .types()
            .iter()
            .find_map(|(id, value)| (value == wanted).then_some(id))
            .ok_or_else(|| vec![diagnostic("verified CoreIR type was not interned")])
    }
}
