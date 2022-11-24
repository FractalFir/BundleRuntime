use crate::codegen_ir::*;
use crate::cil::*;
pub (crate) struct CompiledMethod{
    code:Vec<u8>,
}
//fn compile_method(met:&Method)->CompiledMethod;
#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;
#[cfg(not(target_arch = "x86_64"))]
compile_error!("Architecture not supported.");

