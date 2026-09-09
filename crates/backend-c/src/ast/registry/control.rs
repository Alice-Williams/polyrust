//! Control and allocation identities do not themselves prove flow or ranges.

use super::{CDeclarationKey, CRegistry, CRegistryError, CScopeRef, identity::Identity};

macro_rules! control_reference {
    ($name:ident, $method:ident, $check:ident, $field:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name {
            identity: Identity,
            scope: CScopeRef,
        }
        impl $name {
            pub fn key(&self) -> &CDeclarationKey {
                &self.identity.key
            }
            pub fn scope(&self) -> &CScopeRef {
                &self.scope
            }
        }
        impl CRegistry {
            pub fn $method(
                &mut self,
                scope: &CScopeRef,
                key: CDeclarationKey,
            ) -> Result<$name, CRegistryError> {
                self.check_lexical_scope(scope)?;
                if self
                    .$field
                    .iter()
                    .any(|old| old.scope.function() == scope.function() && old.key() == &key)
                {
                    return Err(CRegistryError::DuplicateRegistration);
                }
                let value = $name {
                    identity: Identity::new(&self.scope, key),
                    scope: scope.clone(),
                };
                self.$field.insert(value.clone());
                Ok(value)
            }
            pub fn $check(&self, scope: &CScopeRef, value: &$name) -> Result<(), CRegistryError> {
                self.check_lexical_scope(scope)?;
                self.check_scope(&value.identity.scope)?;
                if scope.function() != value.scope.function() {
                    return Err(CRegistryError::WrongOwner);
                }
                if self.$field.contains(value) {
                    Ok(())
                } else {
                    Err(CRegistryError::UnregisteredReference)
                }
            }
        }
    };
}

control_reference!(CLoopRef, register_loop, check_loop, loops);
control_reference!(CSwitchRef, register_switch, check_switch, switches);
control_reference!(
    CCleanupExitRef,
    register_cleanup_exit,
    check_cleanup_exit,
    cleanup_exits
);
