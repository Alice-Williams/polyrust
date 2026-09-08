//! Independent expected prototypes and distinguishable argument trees.
use super::*;
use crate::dialect::{CHeader, CKnownCall, CKnownCallForm, CSystemLibrary};

pub(super) struct Row {
    pub call: CKnownCall,
    pub spelling: &'static str,
    pub header: CHeader,
    pub library: Option<CSystemLibrary>,
    pub form: CKnownCallForm,
    pub result: Option<CObjectType>,
    pub parameters: Vec<CObjectType>,
}

pub(super) fn rows() -> Vec<Row> {
    use CKnownCall as K;
    let scalar = CObjectType::scalar;
    let size = scalar(CScalarType::Size);
    let double = scalar(CScalarType::F64);
    let int = scalar(CScalarType::Int);
    let writable = CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified));
    let readable = CObjectType::pointer(CPointerTarget::Void(CConstness::Const));
    let stream = CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::known(
        CKnownObject::File,
    ))));
    let function = CKnownCallForm::Function;
    let mac = CKnownCallForm::DoubleMacro;
    let math = Some(CSystemLibrary::Math);
    vec![
        (
            K::Allocate,
            "malloc",
            CHeader::Stdlib,
            None,
            function,
            Some(writable.clone()),
            vec![size.clone()],
        ),
        (
            K::Release,
            "free",
            CHeader::Stdlib,
            None,
            function,
            None,
            vec![writable.clone()],
        ),
        (
            K::CopyBytes,
            "memcpy",
            CHeader::String,
            None,
            function,
            Some(writable.clone()),
            vec![writable, readable.clone(), size.clone()],
        ),
        (
            K::CompareBytes,
            "memcmp",
            CHeader::String,
            None,
            function,
            Some(int.clone()),
            vec![readable.clone(), readable.clone(), size.clone()],
        ),
        (
            K::FloatRemainder,
            "fmod",
            CHeader::Math,
            math,
            function,
            Some(double.clone()),
            vec![double.clone(), double.clone()],
        ),
        (
            K::FloatTruncate,
            "trunc",
            CHeader::Math,
            math,
            function,
            Some(double.clone()),
            vec![double.clone()],
        ),
        (
            K::IsNan,
            "isnan",
            CHeader::Math,
            None,
            mac,
            Some(int.clone()),
            vec![double.clone()],
        ),
        (
            K::SignBit,
            "signbit",
            CHeader::Math,
            None,
            mac,
            Some(int.clone()),
            vec![double],
        ),
        (
            K::WriteBytes,
            "fwrite",
            CHeader::Stdio,
            None,
            function,
            Some(size.clone()),
            vec![readable, size.clone(), size, stream.clone()],
        ),
        (
            K::StreamError,
            "ferror",
            CHeader::Stdio,
            None,
            function,
            Some(int),
            vec![stream],
        ),
    ]
    .into_iter()
    .map(
        |(call, spelling, header, library, form, result, parameters)| Row {
            call,
            spelling,
            header,
            library,
            form,
            result,
            parameters,
        },
    )
    .collect()
}

impl Row {
    pub fn signature(&self) -> CFunctionType {
        CFunctionType::new(
            self.result.clone().map_or(CReturnType::Void, |ty| {
                CReturnType::Value(CReturnValue::new(ty).unwrap())
            }),
            self.parameters
                .iter()
                .cloned()
                .map(|ty| CParameterType::new(ty).unwrap())
                .collect(),
        )
    }
    pub fn arguments(&self, ast: &CExpressions<'_>) -> Vec<CValue> {
        self.parameters
            .iter()
            .enumerate()
            .map(|(index, ty)| argument(ast, ty, index))
            .collect()
    }
    pub fn call(
        &self,
        ast: &CExpressions<'_>,
        callable: CCallable,
        arguments: Vec<CValue>,
    ) -> Result<CCall, CExpressionError> {
        if self.result.is_none() {
            Ok(ast.call_effect(callable, arguments)?.call().clone())
        } else {
            let value = ast.call_value(callable, arguments)?;
            let CValueKind::Call(call) = value.kind() else {
                panic!("value call")
            };
            assert_eq!(Some(value.ty()), self.result.as_ref());
            Ok(call.clone())
        }
    }
}

pub(super) fn argument(ast: &CExpressions<'_>, ty: &CObjectType, index: usize) -> CValue {
    let number = ast
        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(index as u64 + 1)))
        .unwrap();
    match ty.kind() {
        CObjectTypeKind::Scalar(CScalarType::Size) => number,
        CObjectTypeKind::Scalar(CScalarType::F64) => {
            ast.numeric_conversion(CScalarType::F64, number).unwrap()
        }
        CObjectTypeKind::Pointer(_) => {
            let null = ast
                .literal(CLiteral::NullPointer(
                    CNullPointer::new(ty.clone()).unwrap(),
                ))
                .unwrap();
            ast.conditional(
                ast.literal(CLiteral::Bool(index.is_multiple_of(2)))
                    .unwrap(),
                null.clone(),
                null,
            )
            .unwrap()
        }
        _ => panic!("unexpected known parameter"),
    }
}
