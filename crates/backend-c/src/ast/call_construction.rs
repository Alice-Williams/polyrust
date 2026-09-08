//! Local call shape checks; 02D independently proves effects and provenance.

use super::{
    CCall, CCallable, CCallableContractRef, CCallableKind, CEffect, CExpressionError as E,
    CExpressions, CFunctionRef, CMemberBinding, CObjectTypeKind, CPlaceKind, CPointerTarget,
    CRegistryError, CReturnType, CValue, CValueKind,
};

impl CExpressions<'_> {
    pub fn direct(&self, function: CFunctionRef) -> Result<CCallable, E> {
        self.registry.check_function(&function)?;
        Ok(CCallable {
            brand: self.registry.expression_brand(),
            kind: CCallableKind::Direct,
            function,
        })
    }

    pub fn indirect(
        &self,
        pointer: CValue,
        contract_function: CFunctionRef,
    ) -> Result<CCallable, E> {
        self.check_value(&pointer)?;
        self.registry.check_function(&contract_function)?;
        let CObjectTypeKind::Pointer(CPointerTarget::Function(signature)) = pointer.ty().kind()
        else {
            return Err(E::ExpectedFunctionPointer);
        };
        if !self
            .registry
            .signatures_match(signature, contract_function.signature())?
        {
            return Err(E::TypeMismatch);
        }
        if let Some(actual) = structural_contract(&pointer)
            && actual != contract_function.contract()
        {
            return Err(CRegistryError::CallableContractMismatch.into());
        }
        Ok(CCallable {
            brand: self.registry.expression_brand(),
            kind: CCallableKind::Indirect(Box::new(pointer)),
            function: contract_function,
        })
    }

    fn call(&self, callable: CCallable, arguments: Vec<CValue>) -> Result<CCall, E> {
        if callable.brand != self.registry.expression_brand() {
            return Err(CRegistryError::CrossRegistry.into());
        }
        let parameters = callable.signature().parameters();
        if parameters.len() != arguments.len() {
            return Err(E::ArityMismatch {
                expected: parameters.len(),
                actual: arguments.len(),
            });
        }
        for (argument, parameter) in arguments.iter().zip(parameters) {
            self.check_value(argument)?;
            if !self.registry.types_match(argument.ty(), parameter.ty())? {
                return Err(E::TypeMismatch);
            }
        }
        Ok(CCall {
            callable,
            arguments,
        })
    }

    pub fn call_value(&self, callable: CCallable, arguments: Vec<CValue>) -> Result<CValue, E> {
        let call = self.call(callable, arguments)?;
        let CReturnType::Value(result) = call.callable().signature().return_type() else {
            return Err(E::ExpectedValueCall);
        };
        Ok(self.value(result.ty().clone(), CValueKind::Call(call)))
    }

    pub fn call_effect(&self, callable: CCallable, arguments: Vec<CValue>) -> Result<CEffect, E> {
        let call = self.call(callable, arguments)?;
        if !matches!(call.callable().signature().return_type(), CReturnType::Void) {
            return Err(E::ExpectedEffectCall);
        }
        Ok(CEffect { call })
    }
}

fn structural_contract(value: &CValue) -> Option<&CCallableContractRef> {
    match value.kind() {
        CValueKind::FunctionAddress(function) => Some(function.contract()),
        CValueKind::Read(place) => match place.kind() {
            CPlaceKind::Member { member, .. } => match member.binding() {
                CMemberBinding::Callable(contract) => Some(contract),
                CMemberBinding::Object => None,
            },
            _ => None,
        },
        _ => None,
    }
}
