//! Closed null/self-move tests and explicit qualifier-preserving conversions.

use super::{
    CConstness, CConversion, CExpressionError as E, CExpressions, CObjectType, CObjectTypeKind,
    CPointerTarget, CPointerTest, CScalarType, CValue, CValueKind,
};

impl CExpressions<'_> {
    pub fn numeric_conversion(
        &self,
        destination: CScalarType,
        operand: CValue,
    ) -> Result<CValue, E> {
        self.arithmetic_type(&operand)?;
        Ok(self.value(
            CObjectType::scalar(destination),
            CValueKind::Convert {
                conversion: CConversion::Numeric(destination),
                operand: Box::new(operand),
            },
        ))
    }

    pub fn add_const(&self, destination: CObjectType, operand: CValue) -> Result<CValue, E> {
        self.check_value(&operand)?;
        self.registry.check_type(&destination)?;
        let ty = destination.canonical().without_top_level_const();
        let valid = match (operand.ty().kind(), ty.kind()) {
            (
                CObjectTypeKind::Pointer(CPointerTarget::Object(from)),
                CObjectTypeKind::Pointer(CPointerTarget::Object(to)),
            ) => {
                pointee_constness(from) == CConstness::Unqualified
                    && pointee_constness(to) == CConstness::Const
                    && same_unqualified_pointee(self.registry, from, to)?
            }
            (
                CObjectTypeKind::Pointer(CPointerTarget::Void(CConstness::Unqualified)),
                CObjectTypeKind::Pointer(CPointerTarget::Void(CConstness::Const)),
            ) => true,
            _ => false,
        };
        if !valid {
            return Err(E::InvalidPointerConversion);
        }
        Ok(self.value(
            ty,
            CValueKind::Convert {
                conversion: CConversion::AddConst(destination),
                operand: Box::new(operand),
            },
        ))
    }

    pub fn object_to_void(&self, destination: CObjectType, operand: CValue) -> Result<CValue, E> {
        self.check_value(&operand)?;
        self.registry.check_type(&destination)?;
        let ty = destination.canonical().without_top_level_const();
        let valid = match (operand.ty().kind(), ty.kind()) {
            (
                CObjectTypeKind::Pointer(CPointerTarget::Object(from)),
                CObjectTypeKind::Pointer(CPointerTarget::Void(to)),
            ) => pointee_constness(from) == CConstness::Unqualified || *to == CConstness::Const,
            _ => false,
        };
        if !valid {
            return Err(E::InvalidPointerConversion);
        }
        Ok(self.value(
            ty,
            CValueKind::Convert {
                conversion: CConversion::ObjectToVoid(destination),
                operand: Box::new(operand),
            },
        ))
    }

    pub fn pointer_test(&self, test: CPointerTest) -> Result<CValue, E> {
        match &test {
            CPointerTest::IsNull(value) | CPointerTest::IsNonNull(value) => {
                self.check_value(value)?;
                if !matches!(value.ty().kind(), CObjectTypeKind::Pointer(_)) {
                    return Err(super::CTypeError::ExpectedPointer.into());
                }
            }
            CPointerTest::SameSlot { left, right } => {
                self.check_value(left)?;
                self.check_value(right)?;
                if !is_slot_type(left.ty()) || !self.registry.types_match(left.ty(), right.ty())? {
                    return Err(E::ExpectedOwningSlotPointer);
                }
            }
        }
        Ok(self.value(
            CObjectType::scalar(CScalarType::Int),
            CValueKind::PointerTest(test),
        ))
    }
}

fn is_slot_type(ty: &CObjectType) -> bool {
    let CObjectTypeKind::Pointer(CPointerTarget::Object(slot)) = ty.kind() else {
        return false;
    };
    slot.constness() == CConstness::Unqualified
        && match slot.kind() {
            CObjectTypeKind::Pointer(CPointerTarget::Object(target)) => {
                pointee_constness(target) == CConstness::Unqualified
                    && target.require_storable().is_ok()
            }
            CObjectTypeKind::Pointer(CPointerTarget::Void(qualifier)) => {
                *qualifier == CConstness::Unqualified
            }
            _ => false,
        }
}

// Array qualification lives on its element, but a nested pointer's pointee
// qualification is a different level and must remain unchanged.
fn same_unqualified_pointee(
    registry: &super::CRegistry,
    mut left: &CObjectType,
    mut right: &CObjectType,
) -> Result<bool, super::CRegistryError> {
    while let (
        CObjectTypeKind::Array {
            element: a,
            length: x,
        },
        CObjectTypeKind::Array {
            element: b,
            length: y,
        },
    ) = (left.kind(), right.kind())
    {
        if x != y {
            return Ok(false);
        }
        left = a;
        right = b;
    }
    registry.types_match(
        &left.clone().without_top_level_const(),
        &right.clone().without_top_level_const(),
    )
}

fn pointee_constness(mut ty: &CObjectType) -> CConstness {
    while let CObjectTypeKind::Array { element, .. } = ty.kind() {
        ty = element;
    }
    ty.constness()
}
