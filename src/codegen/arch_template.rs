use crate::codegen::CompiledMethod;
use crate::codegen_ir::MethodIR;
pub (crate) fn compile_method(method:&MethodIR,path:&str)->CompiledMethod{
    let mut code = Vec::new();
    for op in method.ops.iter(){
        match op{
            _=>todo!("operation {op:?} is not supported by native codegen on ARCHITECTURE_NAME."),
        }
    }
    CompiledMethod{code:code.into(),path.to_owned()}
}
