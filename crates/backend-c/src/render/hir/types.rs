use super::super::CStdType;
use super::Writer;
use crate::ast::{CConstness, CObjectType, CObjectTypeKind, CPointerTarget, CScalarType};

impl Writer<'_> {
    pub(super) fn scalar(&self, scalar: CScalarType) -> &str {
        match scalar {
            CScalarType::I32 => self.names.standards[&CStdType::I32].as_str(),
            CScalarType::Int => "int",
            CScalarType::Bool => "_Bool",
            _ => unreachable!("checked scalar profile"),
        }
    }
    pub(super) fn declarator(&self, ty: &CObjectType, name: &str) -> String {
        let qualifier = if ty.constness() == CConstness::Const {
            "const "
        } else {
            ""
        };
        match ty.kind() {
            CObjectTypeKind::Scalar(scalar) => {
                format!("{qualifier}{} {name}", self.scalar(*scalar))
                    .trim_end()
                    .into()
            }
            CObjectTypeKind::Struct(record) => format!(
                "{qualifier}struct {} {name}",
                self.names.types[record].as_str()
            )
            .trim_end()
            .into(),
            CObjectTypeKind::Pointer(CPointerTarget::Object(pointee)) => {
                self.declarator(pointee, &format!("*{qualifier}{name}"))
            }
            _ => unreachable!("checked declarator profile"),
        }
    }
}
