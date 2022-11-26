use crate::codegen::CompiledMethod;
use crate::codegen_ir::MethodIR;
use crate::codegen_ir::{IrVar,IrOp};
fn modrm_adress_register(r:&IrVar,b:&IrVar)->u8{
    let r = (var_as_reg(r))%8;
    let b = (var_as_reg(b))%8;
    (0b11<<6)|(r<<3)|b
}
fn var_as_reg(var:&IrVar)->u8{
    match var{
        IrVar::IReg(var)=>{
            if *var > 0xF{panic!("x86_64 has only 16 registers, but instruction tried to acces {var}.")};
            CALL_REG_ORDER[*var as usize]
        }
    }
}
const CALL_REG_ORDER:[u8;16] = [7,6,3,2,8,9,//rdi, rsi, rdx, rcx, r8,r9 
0,1,4,5,10,11,12,13,14,15]; // and after that normal(registers changed as a result of calling convention 
fn create_rex_prefix(bit_pattern:u8,is_wide:bool,r:&IrVar,b:&IrVar)->u8{
    let r = (var_as_reg(r) > 7) as u8;
    let x = 0; // Unused for now.
    let b = (var_as_reg(b) >  7) as u8;
    let bit_pattern:u8 = (bit_pattern);
    (bit_pattern<<4) | (is_wide as u8)<<3 | (r<<2) | (x<<1) | b
}
pub (crate) fn compile_method(method:&MethodIR,path:&str)->CompiledMethod{
    let mut code = Vec::new();
    for op in method.ops.iter(){
        match op{
            IrOp::Mul(multiplicant,multiplier)=>{
                let rex = create_rex_prefix(0b0100,true,multiplier,multiplicant);
                let mod_rm = modrm_adress_register(multiplier,multiplicant);
                code.extend_from_slice(&[rex,0x0F,0xAF,mod_rm]);
            },
            IrOp::Add(to,increment)=>{
                let rex = create_rex_prefix(0b0100,true,increment,to);
                let mod_rm = modrm_adress_register(increment,to);
                code.extend_from_slice(&[rex,0x01,mod_rm]);
            },
            IrOp::Sub(to,increment)=>{
                let rex = create_rex_prefix(0b0100,true,increment,to);
                let mod_rm = modrm_adress_register(increment,to);
                code.extend_from_slice(&[rex,0x29,mod_rm]);
            },
            IrOp::Return(reg)=>{
                let to = IrVar::IReg(6); //rax
                if(*reg != to){
                    let rex = create_rex_prefix(0b0100,true,reg,&to);
                    let mod_rm = modrm_adress_register(reg,&to);
                    code.extend_from_slice(&[rex,0x89,mod_rm]);
                }
                code.push(0xc3);
            },
            _=>todo!("operation {op:?} is not supported by native codegen."),
        }
    }
    CompiledMethod{code:code.into(),path:path.to_owned(),arg_count:method.arg_count}
}
