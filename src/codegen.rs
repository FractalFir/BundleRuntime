pub enum CodeGenOp{
    Add,
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
                CodeGenOp::Add=>todo!(),
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
    use crate::codegen::{MethodStub,AssemblyStub,Assembly,CodeGenOp};
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
    
}
