use serde::Serialize;

macro_rules! core_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub(crate) u32);

        impl $name {
            pub const fn index(self) -> usize {
                self.0 as usize
            }

            pub(crate) fn from_index(index: usize) -> Self {
                Self(u32::try_from(index).expect("CoreIR arena exceeds u32"))
            }
        }
    };
}

core_id!(CoreTypeId);
core_id!(CoreConstantId);
core_id!(CoreAliasId);
core_id!(CoreRecordId);
core_id!(CoreEnumId);
core_id!(CoreVariantId);
core_id!(CoreFieldId);
core_id!(CoreInterfaceId);
core_id!(CoreInterfaceMethodId);
core_id!(CoreImplementationId);
core_id!(CoreImplementationMethodId);
core_id!(CoreFunctionId);
core_id!(CoreTestId);
core_id!(CoreLocalId);
core_id!(CoreExprId);
core_id!(CoreBlockId);
