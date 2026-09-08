//! Registered allocation/adapter paths, not caller-authored ownership evidence.

use super::{
    CAllocationRef, CConstness, CConversion, CExpressionError as E, CExpressions,
    CInterfaceAdapterRef, CObjectType, CObjectTypeKind, CPointerTarget, CValue, CValueKind,
};

impl CExpressions<'_> {
    pub fn allocation_restore(
        &self,
        allocation: CAllocationRef,
        operand: CValue,
    ) -> Result<CValue, E> {
        self.check_value(&operand)?;
        self.registry
            .check_allocation(allocation.scope(), &allocation)?;
        if !matches!(
            operand.ty().kind(),
            CObjectTypeKind::Pointer(CPointerTarget::Void(CConstness::Unqualified))
        ) {
            return Err(E::InvalidPointerConversion);
        }
        let ty = CObjectType::pointer(CPointerTarget::Object(Box::new(
            allocation.object_type().canonical(),
        )));
        Ok(self.value(
            ty,
            CValueKind::Convert {
                conversion: CConversion::AllocationRestore(Box::new(allocation)),
                operand: Box::new(operand),
            },
        ))
    }

    pub fn adapter_erase(
        &self,
        adapter: CInterfaceAdapterRef,
        operand: CValue,
    ) -> Result<CValue, E> {
        self.check_value(&operand)?;
        self.registry.check_interface_adapter(&adapter)?;
        let CObjectTypeKind::Pointer(CPointerTarget::Object(record)) = operand.ty().kind() else {
            return Err(E::InvalidPointerConversion);
        };
        let expected = CObjectType::structure(adapter.witness().record().clone());
        if !self
            .registry
            .types_match(&(**record).clone().without_top_level_const(), &expected)?
        {
            return Err(E::InvalidPointerConversion);
        }
        let ty = CObjectType::pointer(CPointerTarget::Void(record.constness()));
        Ok(self.value(
            ty,
            CValueKind::Convert {
                conversion: CConversion::AdapterErase(Box::new(adapter)),
                operand: Box::new(operand),
            },
        ))
    }

    pub fn adapter_restore(
        &self,
        adapter: CInterfaceAdapterRef,
        operand: CValue,
    ) -> Result<CValue, E> {
        self.check_value(&operand)?;
        self.registry.check_interface_adapter(&adapter)?;
        let CObjectTypeKind::Pointer(CPointerTarget::Void(qualifier)) = operand.ty().kind() else {
            return Err(E::InvalidPointerConversion);
        };
        let record = CObjectType::structure(adapter.witness().record().clone())
            .with_constness(*qualifier)?;
        let ty = CObjectType::pointer(CPointerTarget::Object(Box::new(record)));
        Ok(self.value(
            ty,
            CValueKind::Convert {
                conversion: CConversion::AdapterRestore(Box::new(adapter)),
                operand: Box::new(operand),
            },
        ))
    }
}
