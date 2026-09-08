//! Typed storage paths; construction does not establish initialized/range facts.

use super::{
    CAggregateRef, CConstness, CExpressionError as E, CExpressions, CIndexBase, CLocalRef,
    CMemberRef, CObjectRef, CObjectType, CObjectTypeKind, CParameterRef, CPlace, CPlaceKind,
    CPointerTarget, CValue,
};

impl CExpressions<'_> {
    fn place(&self, ty: CObjectType, kind: CPlaceKind) -> CPlace {
        CPlace {
            brand: self.registry.expression_brand(),
            ty: ty.canonical(),
            kind,
        }
    }

    pub fn local(&self, value: CLocalRef) -> Result<CPlace, E> {
        self.registry
            .check_local(value.scope().function(), &value)?;
        Ok(self.place(value.ty().clone(), CPlaceKind::Local(value)))
    }

    pub fn parameter(&self, value: CParameterRef) -> Result<CPlace, E> {
        self.registry.check_parameter(value.function(), &value)?;
        Ok(self.place(value.ty().clone(), CPlaceKind::Parameter(value)))
    }

    pub fn global(&self, value: CObjectRef) -> Result<CPlace, E> {
        self.registry.check_object(&value)?;
        Ok(self.place(value.ty().clone(), CPlaceKind::Global(value)))
    }

    pub fn member(&self, base: CPlace, member: CMemberRef) -> Result<CPlace, E> {
        self.check_place(&base)?;
        let owner = match base.ty().kind() {
            CObjectTypeKind::Struct(value) => CAggregateRef::Struct(value.clone()),
            CObjectTypeKind::Union(value) => CAggregateRef::Union(value.clone()),
            _ => return Err(E::ExpectedAggregate),
        };
        self.registry.check_member(&owner, &member)?;
        let mut ty = member.ty().canonical();
        if base.ty().constness() == CConstness::Const {
            ty = qualify_const(ty)?;
        }
        Ok(self.place(
            ty,
            CPlaceKind::Member {
                base: Box::new(base),
                member,
            },
        ))
    }

    pub fn dereference(&self, pointer: CValue) -> Result<CPlace, E> {
        self.check_value(&pointer)?;
        let CObjectTypeKind::Pointer(CPointerTarget::Object(target)) = pointer.ty().kind() else {
            return Err(E::ExpectedObjectPointer);
        };
        Ok(self.place(
            (**target).clone(),
            CPlaceKind::Dereference(Box::new(pointer)),
        ))
    }

    pub fn index(&self, base: CIndexBase, index: CValue) -> Result<CPlace, E> {
        if self.arithmetic_type(&index)?.integer_promotion().is_none() {
            return Err(super::COperatorError::ExpectedInteger.into());
        }
        let ty = match &base {
            CIndexBase::Array(place) => {
                self.check_place(place)?;
                let CObjectTypeKind::Array { element, .. } = place.ty().kind() else {
                    return Err(E::ExpectedArray);
                };
                (**element).clone()
            }
            CIndexBase::Pointer(value) => {
                self.check_value(value)?;
                let CObjectTypeKind::Pointer(CPointerTarget::Object(target)) = value.ty().kind()
                else {
                    return Err(E::ExpectedObjectPointer);
                };
                target.require_storable()?;
                (**target).clone()
            }
        };
        Ok(self.place(
            ty,
            CPlaceKind::Index {
                base,
                index: Box::new(index),
            },
        ))
    }
}

fn qualify_const(ty: CObjectType) -> Result<CObjectType, E> {
    if let CObjectTypeKind::Array { element, length } = ty.kind() {
        Ok(CObjectType::array(
            qualify_const((**element).clone())?,
            *length,
        )?)
    } else {
        Ok(ty.with_constness(CConstness::Const)?)
    }
}
