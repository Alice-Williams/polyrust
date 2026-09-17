//! Only certificate-derived closed identities become native link metadata.
use super::ApiManifest;
use portable_backend_c::dialect::CSystemLibrary;
impl ApiManifest {
    pub(super) fn write_system_libraries(&self, text: &mut String) {
        if self.system_libraries.is_empty() {
            return;
        }
        text.push_str(",\"system_libraries\":[");
        for (index, library) in self.system_libraries.iter().enumerate() {
            if index != 0 {
                text.push(',');
            }
            text.push_str(match library {
                CSystemLibrary::Math => "\"m\"",
            });
        }
        text.push(']');
    }
}
