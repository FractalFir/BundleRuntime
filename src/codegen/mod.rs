use crate::codegen_ir::*;
use crate::cil::*;
pub (crate) struct CompiledMethod{
    code:Vec<u8>,
    arg_count:usize,
    path:String,
}
impl CompiledMethod{
    /// Gets the max space needed for writing this method(in future including alignment)
    pub(crate) fn get_max_space(&self)->usize{
        self.code.len()
    }
    pub(crate) fn get_size(&self)->usize{
        self.code.len()
    }
    pub(crate) fn align(&self,offset:usize)->usize{offset}
    pub(crate) fn compile(method:&MethodIR,path:&str)->Self{compile_method(method,path)}
}
//fn compile_method(met:&Method)->CompiledMethod;
#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
use x86_64::compile_method;
#[cfg(not(target_arch = "x86_64"))]
compile_error!("Architecture not supported.");
use crate::mem_mgr::ExecutableMemory;
use std::collections::HashMap;
struct MethodInfo{
    offset:usize,
    arg_count:usize,
}
impl MethodInfo{
    fn new(offset:usize,arg_count:usize)->Self{Self{offset,arg_count}}
}
pub struct Assembly{
    exec_mem:ExecutableMemory,
    method_info:HashMap<String,MethodInfo>,
}
impl Assembly{
    //TODO: make private?
    pub fn from_raw(methods:&[MethodIR])->Self{
        let mut total_size:usize = 0;
        let mut compiled_methods = Vec::with_capacity(methods.len());
        for method in methods{
            let compiled = CompiledMethod::compile(method,"TODO");
            total_size += compiled.get_max_space();
            compiled_methods.push(compiled);
        }
        let exec_mem = ExecutableMemory::new(total_size);
        let mut offset = 0;
        let mut method_info = HashMap::with_capacity(methods.len());
        for method in compiled_methods{
          offset = method.align(offset);
          let beg = offset;
          let size = method.get_size();
          let slice = exec_mem.get_mut_slice_at(offset,size);
          slice.copy_from_slice(&method.code);
          offset += size;
          method_info.insert(method.path,MethodInfo::new(beg,method.arg_count));
        }
        Self{exec_mem,method_info}
    }
}
impl Assembly{
    pub fn get_method<'a,Args,Ret>(&'a self,key:&str)->Option<fn(Args)->Ret>{
        let mi = self.method_info.get(key)?;
        let ptr = self.exec_mem.get_ptr(mi.offset);
        unsafe{std::mem::transmute(ptr)}
    }
}
#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn identity(){
        let identity = MethodIR::new(&[CILOp::LdArg(0),CILOp::Ret],1);
        let asm = Assembly::from_raw(&[identity]);
        let identity:fn(u64)->u64 = asm.get_method("TODO").unwrap();
        for i in 0..64{
            let arg = 1<<i;
            let res = identity(arg);
            assert!(res == arg,"{res} != {arg}");
        }
    }
    #[test]
    fn sqr_mag(){
        let sqr_mag = MethodIR::new(&[CILOp::LdArg(0), CILOp::LdArg(0), CILOp::Mul, CILOp::LdArg(1), CILOp::LdArg(1), CILOp::Mul, CILOp::Add, CILOp::Ret],2);
        let asm = Assembly::from_raw(&[sqr_mag]);
        let sqr_mag:fn((u64,u64))->u64 = asm.get_method("TODO").unwrap();
        for x in 0..0xFF{
            for y in 0..0xFF{
                let res = sqr_mag((x,y));
                assert!(x*x + y*y == res,"{res}");
            }
        }
    }
}
