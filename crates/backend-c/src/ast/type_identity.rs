//! Private compatibility keys for storage, never replacements for source origins.
use super::{
    CConstness, CFunctionType, CObjectType, CObjectTypeKind as T, CParameterType, CPointerTarget,
    CRegistry, CRegistryError as E, CReturnType, CReturnValue,
};

impl CRegistry {
    pub(crate) fn pointee_storage_identity(&self, ty: &CObjectType) -> Result<CObjectType, E> {
        // Authenticate aliases and callback declared types before normalizing.
        self.check_type(ty)?;
        object(ty, true)
    }
}

fn object(ty: &CObjectType, immediate: bool) -> Result<CObjectType, E> {
    let ty = ty.canonical();
    let normalized = match ty.kind() {
        T::Scalar(value) => CObjectType::scalar(value.abi_identity()),
        T::Known(value) => CObjectType::known(*value),
        T::Struct(value) => CObjectType::structure(value.clone()),
        T::Union(value) => CObjectType::union(value.clone()),
        T::Enum(value) => CObjectType::enumeration(value.clone()),
        T::Array { element, length } => CObjectType::array(object(element, immediate)?, *length)
            .map_err(E::InvalidObjectType)?,
        T::Pointer(target) => CObjectType::pointer(match target {
            CPointerTarget::Void(qualifier) => CPointerTarget::Void(*qualifier),
            CPointerTarget::Object(target) => {
                CPointerTarget::Object(Box::new(object(target, false)?))
            }
            CPointerTarget::Function(signature) => {
                CPointerTarget::Function(Box::new(function(signature)?))
            }
        }),
        T::Typedef(_) => unreachable!("authenticated canonical type expands structural aliases"),
    };
    normalized
        .with_constness(if immediate {
            CConstness::Unqualified
        } else {
            ty.constness()
        })
        .map_err(E::InvalidObjectType)
}

fn function(signature: &CFunctionType) -> Result<CFunctionType, E> {
    let result = match signature.return_type() {
        CReturnType::Void => CReturnType::Void,
        CReturnType::Value(value) => CReturnType::Value(
            CReturnValue::new(object(value.ty(), false)?).map_err(E::InvalidObjectType)?,
        ),
    };
    let parameters = signature
        .parameters()
        .iter()
        .map(|value| CParameterType::new(object(value.ty(), false)?).map_err(E::InvalidObjectType))
        .collect::<Result<_, E>>()?;
    Ok(CFunctionType::new(result, parameters))
}
