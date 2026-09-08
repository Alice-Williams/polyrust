//! Shape/type construction. Scope, range and initialization remain verifier work.

use super::{
    CBinaryOperator, CEnumeratorRef, CExpressionError as E, CFunctionRef, CLiteral, CObjectType,
    CObjectTypeKind, CPlace, CPointerTarget, CRegistry, CRegistryError, CScalarType,
    CUnaryOperator, CValue, CValueKind,
};

pub struct CExpressions<'a> {
    pub(super) registry: &'a CRegistry,
}

impl<'a> CExpressions<'a> {
    pub fn known_constant(&self, value: super::CKnownConstant) -> CValue {
        self.value(value.ty(), CValueKind::KnownConstant(value))
    }

    pub const fn new(registry: &'a CRegistry) -> Self {
        Self { registry }
    }

    pub(super) fn value(&self, ty: CObjectType, kind: CValueKind) -> CValue {
        CValue {
            brand: self.registry.expression_brand(),
            ty,
            kind,
        }
    }

    pub(super) fn check_value(&self, value: &CValue) -> Result<(), E> {
        if value.brand == self.registry.expression_brand() {
            Ok(())
        } else {
            Err(CRegistryError::CrossRegistry.into())
        }
    }

    pub(super) fn check_place(&self, place: &CPlace) -> Result<(), E> {
        if place.brand == self.registry.expression_brand() {
            Ok(())
        } else {
            Err(CRegistryError::CrossRegistry.into())
        }
    }

    pub(super) fn arithmetic_type(&self, value: &CValue) -> Result<CScalarType, E> {
        self.check_value(value)?;
        match value.ty().kind() {
            CObjectTypeKind::Scalar(value) => Ok(*value),
            CObjectTypeKind::Enum(value) => {
                let values = self.registry.enumerators(value)?.ok_or(E::IncompleteEnum)?;
                Ok(if values.iter().any(|value| value.value() < 0) {
                    CScalarType::Int
                } else {
                    CScalarType::U32
                })
            }
            _ => Err(E::ExpectedArithmetic),
        }
    }

    pub(super) fn check_bool(&self, value: &CValue) -> Result<(), E> {
        self.check_value(value)?;
        if value.ty() == &CObjectType::scalar(CScalarType::Bool) {
            Ok(())
        } else {
            Err(E::ExpectedBool)
        }
    }

    pub fn literal(&self, literal: CLiteral) -> Result<CValue, E> {
        if let CLiteral::NullPointer(value) = &literal {
            self.registry.check_type(value.declared_type())?;
        }
        Ok(self.value(literal.ty(), CValueKind::Literal(literal)))
    }

    pub fn enumerator(&self, value: CEnumeratorRef) -> Result<CValue, E> {
        self.registry.check_enumerator(&value)?;
        Ok(self.value(
            CObjectType::scalar(CScalarType::Int),
            CValueKind::Enumerator(value),
        ))
    }

    pub fn function_address(&self, value: CFunctionRef) -> Result<CValue, E> {
        self.registry.check_function(&value)?;
        let ty = CObjectType::pointer(CPointerTarget::Function(Box::new(
            value.signature().clone(),
        )));
        Ok(self.value(ty, CValueKind::FunctionAddress(value)))
    }

    pub fn read(&self, place: CPlace) -> Result<CValue, E> {
        self.check_place(&place)?;
        if place.ty().is_array() {
            return Err(E::ArrayReadRequiresExplicitAddress);
        }
        place.ty().require_storable()?;
        let ty = place.ty().clone().without_top_level_const();
        Ok(self.value(ty, CValueKind::Read(Box::new(place))))
    }

    pub fn address_of(&self, place: CPlace) -> Result<CValue, E> {
        self.check_place(&place)?;
        let ty = CObjectType::pointer(CPointerTarget::Object(Box::new(place.ty().clone())));
        Ok(self.value(ty, CValueKind::AddressOf(Box::new(place))))
    }

    pub fn unary(&self, operator: CUnaryOperator, operand: CValue) -> Result<CValue, E> {
        let result = operator.result_type(self.arithmetic_type(&operand)?)?;
        Ok(self.value(
            CObjectType::scalar(result),
            CValueKind::Unary {
                operator,
                operand: Box::new(operand),
            },
        ))
    }

    pub fn binary(
        &self,
        operator: CBinaryOperator,
        left: CValue,
        right: CValue,
    ) -> Result<CValue, E> {
        let result =
            operator.result_type(self.arithmetic_type(&left)?, self.arithmetic_type(&right)?)?;
        Ok(self.value(
            CObjectType::scalar(result),
            CValueKind::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
        ))
    }

    pub fn conditional(
        &self,
        condition: CValue,
        then_value: CValue,
        else_value: CValue,
    ) -> Result<CValue, E> {
        self.check_bool(&condition)?;
        self.check_value(&then_value)?;
        self.check_value(&else_value)?;
        if !self
            .registry
            .types_match(then_value.ty(), else_value.ty())?
        {
            return Err(E::TypeMismatch);
        }
        // C's conditional applies integer promotions even for equal narrow types.
        let ty = match then_value.ty().kind() {
            CObjectTypeKind::Scalar(_) | CObjectTypeKind::Enum(_) => CObjectType::scalar(
                self.arithmetic_type(&then_value)?
                    .usual_arithmetic_conversion(self.arithmetic_type(&else_value)?),
            ),
            _ => then_value.ty().clone(),
        };
        Ok(self.value(
            ty,
            CValueKind::Conditional {
                condition: Box::new(condition),
                then_value: Box::new(then_value),
                else_value: Box::new(else_value),
            },
        ))
    }

    pub fn size_of(&self, ty: CObjectType) -> Result<CValue, E> {
        self.registry.check_type(&ty)?;
        ty.require_storable()?;
        Ok(self.value(
            CObjectType::scalar(CScalarType::Size),
            CValueKind::SizeOf(ty),
        ))
    }

    pub fn align_of(&self, ty: CObjectType) -> Result<CValue, E> {
        self.registry.check_type(&ty)?;
        ty.require_storable()?;
        Ok(self.value(
            CObjectType::scalar(CScalarType::Size),
            CValueKind::AlignOf(ty),
        ))
    }
}
