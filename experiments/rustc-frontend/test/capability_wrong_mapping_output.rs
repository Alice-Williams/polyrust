use super::{Builder, LiteralInput, LiteralValues, Mapping};
use crate::c_lower::{Reader, Result};
use portable_backend_c::ast::CPlace;

#[derive(Clone, Copy)]
struct WrongOutput;
impl Mapping for WrongOutput {
    type Capability = LiteralValues;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CPlace;
    fn lower<'tcx>(&self, _: &mut Reader<'tcx>, _: LiteralInput<'tcx>) -> Result<CPlace> {
        Err("deliberately wrong mapping output".into())
    }
}
#[allow(dead_code)]
fn must_not_compile() {
    Builder::new().literal_values(WrongOutput);
}
