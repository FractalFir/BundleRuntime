#[derive(PartialEq)]
struct LocVar(u8);
const REX_W:u8 = 0x48;
struct REX(u8);
fn create_rex_prefix(is_wide:bool,a:&LocVar,b:&LocVar)->u8{
    let is_wide = (is_wide as u8) << 3;
    let mod_rm_a = (a.is_extended() as u8) << 2;
    let mod_rm_b = b.is_extended() as u8;
    0b100<<4 | is_wide | mod_rm_a | mod_rm_b
}
impl LocVar{
    fn into_mod_rm(src:&LocVar,dst:&LocVar)->u8{ 
        if !src.is_reg() || !dst.is_reg(){panic!("Only up to 8 local variables are supported now.")}
        else {0xc0 | src.0 << 3 | dst.0}
    }
    fn is_reg(&self)->bool{self.0 < 16}
    fn is_extended(&self)->bool{self.0 > 7 && self.0 <  16} //Checks if LocVar is extended (registers 8-15)
}
pub enum CodeGenOp{
    //Add(LocVar,LocVar),
    MovI32(LocVar,LocVar),
    //MovI64(LocVar,LocVar),
    Ret,
}
//Link entry: insert_point assembly_id method_id
//Label entry: code_point 
//Field offset entry: insert_point assembly_id class_id field_id
//Link entry: insert_point assembly_id method_id
//"Stub" is a temporary name used until a better name is found.
pub struct MethodStub{
    code:Vec<u8>,
    link_table:Vec<(usize,(u32,u32))>,
    field_offset_table:Vec<(usize,(u32,u32,u32))>,
    label_table:Vec<usize>,
    label_jump_table:Vec<(usize,usize)>,
}
impl MethodStub{
    pub fn new(ops:&[CodeGenOp])->Self{
        let mut nmeth = Self{code:Vec::new(), link_table:Vec::new(), field_offset_table:Vec::new(), label_table:Vec::new(),label_jump_table:Vec::new()};
        nmeth.compile(ops);
        nmeth.finish_codegen();
        nmeth
    }
    //This is architecture specific function. This and only this will be architecture specific!
    fn compile(&mut self,ops:&[CodeGenOp]){
        for op in ops{
            match op{
                CodeGenOp::MovI32(src,dst)=>{
                    self.code.push(create_rex_prefix(true,src,dst)); //Prefix
                    println!("Prefix:{:x}",create_rex_prefix(true,src,dst));
                    self.code.push(0x8b); //Move Register to Register
                    self.code.push(LocVar::into_mod_rm(src,dst)); // Mod/RM byte
                    println!("Mod/RM:{:b}",LocVar::into_mod_rm(src,dst));
                }
                CodeGenOp::Ret=>self.code.push(0o303),
            }
        }
    }
    //Finishes codegen and links internal labels.
    fn finish_codegen(&mut self){}
}
pub struct AssemblyStub{
    methods:Vec<MethodStub>,
}
impl AssemblyStub{
    fn new()->Self{Self{methods:Vec::new()}}
    fn into_assembly(self:Self)->Assembly{
        let size = self.stubs_size();
        let exec_mem = ExecutableMemory::new(size);
        let mut slice = exec_mem.get_mut_slice_at(0,size);
        let mut offset:usize = 0;
        let mut methods = Vec::with_capacity(self.methods.len());
        for method in self.methods{
            methods.push(offset);
            for byte in method.code{
                slice[offset] = byte;
                offset += 1;
            }
        }
        let methods:Box<[usize]> = methods.into();
        Assembly{exec_mem,methods}
    }
    fn stubs_size(&self)->usize{
        return 4095;
    }
}
use crate::mem_mgr::*;
#[derive(Debug)]
pub struct Assembly{
    exec_mem:ExecutableMemory,
    methods:Box<[usize]>,
}
impl Assembly{
    pub unsafe fn get_method<Args,Ret>(&self,id:usize)->fn(Args)->Ret{
        let ptr = self.exec_mem.get_ptr(self.methods[id]);
        unsafe{std::mem::transmute(ptr)}
    }
}
#[cfg(test)]
mod test{
    use crate::codegen::{MethodStub,AssemblyStub,Assembly,CodeGenOp,LocVar};
    #[test]
    fn compile_simplest(){
        let mut asm = AssemblyStub::new();
        let code = [CodeGenOp::Ret];
        let mstub = MethodStub::new(&code);
        asm.methods.push(mstub);
        let asm = asm.into_assembly();
        println!("asm:{asm:?}");
        let met = unsafe{asm.get_method::<(),()>(0)};
    }
    #[test]
    fn execute_simplest(){
        let mut asm = AssemblyStub::new();
        let code = [CodeGenOp::Ret];
        let mstub = MethodStub::new(&code);
        asm.methods.push(mstub);
        let asm = asm.into_assembly();
        println!("asm:{asm:?}");
        let met = unsafe{asm.get_method::<(),()>(0)};
        met(());
    }
    #[test]
    fn execute_identity(){
        let mut asm = AssemblyStub::new();
        let code = [CodeGenOp::MovI32(LocVar(0),LocVar(7)),CodeGenOp::Ret];
        let mstub = MethodStub::new(&code);
        asm.methods.push(mstub);
        let asm = asm.into_assembly();
        println!("asm:{asm:?}");
        let met = unsafe{asm.get_method::<u16,u16>(0)};
        for i in 0..0xFFFF{
            let res = met(i);
            assert!(i == res,"{res} != {i}");
        }
    }
    #[test]
    fn execute_identity_wide_move(){
        let mut asm = AssemblyStub::new();
        //Segfault happens because registers which should be callee saved are overwritten!
        let code = [CodeGenOp::MovI32(LocVar(0),LocVar(1)),CodeGenOp::MovI32(LocVar(1),LocVar(2)),CodeGenOp::MovI32(LocVar(2),LocVar(13)),CodeGenOp::MovI32(LocVar(13),LocVar(7)),CodeGenOp::Ret];
        let mstub = MethodStub::new(&code);
        asm.methods.push(mstub);
        let asm = asm.into_assembly();
        println!("asm:{asm:?}");
        let met = unsafe{asm.get_method::<u16,u16>(0)};
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        //panic!();
        #[inline(never)]
        fn call_met(met:fn(u16)->u16,arg:u16)->u16{
            met(arg)
        }
        for i in 0..0x4{
            let res = call_met(met,i);
            println!("res:{res}");
            std::io::stdout().flush().unwrap();
            //assert!(i == res,"{res} != {i}");
        }
    }
}
