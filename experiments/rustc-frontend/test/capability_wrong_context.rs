use super::{Builder, LiteralInput, LiteralValues, Mapping};
use crate::c_lower::Result;
use portable_backend_c::ast::CValue;

#[derive(Clone, Copy)]
struct WrongContext;
impl Mapping for WrongContext {
    type Capability = LiteralValues;
    type Context<'tcx> = ();
    type Output = CValue;
    fn lower<'tcx>(&self, _: &mut (), _: LiteralInput<'tcx>) -> Result<CValue> {
        Err("deliberately wrong mapping context".into())
    }
}
#[allow(dead_code)]
fn must_not_compile() {
    Builder::new().literal_values(WrongContext);
}
