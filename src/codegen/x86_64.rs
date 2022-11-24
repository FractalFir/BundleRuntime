use crate::codegen::CompiledMethod;
use crate::codegen_ir::Method;
use ux::u4;
use crate::codegen_ir::{IrVar,IrOp};
fn modrm_adress_register(r:&IrVar,b:&IrVar)->u8{
    let r = (u8::from(var_as_reg(r)))%8;
    let b = (u8::from(var_as_reg(b)))%8;
    (0b11<<6)|(r<<3)|b
}
fn var_as_reg(var:&IrVar)->u4{
    match var{
        IrVar::IReg(var)=>{
            if *var > 0xF{panic!("x86_64 has only 16 registers, but instruction tried to acces {var}.")};
            u4::new(*var as u8)
        }
    }
}
fn create_rex_prefix(bit_pattern:u4,is_wide:bool,r:&IrVar,b:&IrVar)->u8{
    let r = (var_as_reg(r) >  u4::new(7)) as u8;
    let x = 0; // Unused for now.
    let b = (var_as_reg(b) >  u4::new(7)) as u8;
    let bit_pattern:u8 = (<u4 as Into<u8>>::into(bit_pattern));
    (bit_pattern<<4) | (is_wide as u8)<<3 | (r<<2) | (x<<1) | b
}
pub (crate) fn compile_method(method:&Method)->CompiledMethod{
    let mut code = Vec::new();
    for op in method.ops.iter(){
        match op{
            IrOp::Mul(multiplicant,multiplier)=>{
                let rex = create_rex_prefix(u4::new(0b0100),true,multiplicant,multiplier);
                let mod_rm = modrm_adress_register(multiplicant,multiplier);
                code.extend_from_slice(&[rex,0x0F,0xAF,mod_rm]);
            },
            IrOp::Add(to,increment)=>{
                let rex = create_rex_prefix(u4::new(0b0100),true,to,increment);
                let mod_rm = modrm_adress_register(to,increment);
                code.extend_from_slice(&[rex,0x01,mod_rm]);
            },
            IrOp::Return(reg)=>{
                let to = IrVar::IReg(0);
                if(*reg != to){
                    let rex = create_rex_prefix(u4::new(0b0100),true,&to,reg);
                    let mod_rm = modrm_adress_register(&to,reg);
                    code.extend_from_slice(&[rex,0x89,mod_rm]);
                }
                code.push(0xc3);
            },
            _=>todo!("operation {op:?} is not supported by native codegen."),
        }
    }
    CompiledMethod{code:code.into()}
}
