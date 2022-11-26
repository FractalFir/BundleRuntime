use crate::codegen_ir::*;
#[derive(Debug)]
pub enum CILOp{
    LdArg(usize),
    Ret,
    Add,
    Sub,
    Mul,
}
impl CILOp{
    pub (crate) fn to_codegen_ir(&self,res:&mut Vec<IrOp>,highest_var:&mut usize){
        match self{
            Self::LdArg(arg)=>{res.push(IrOp::Push(IrVar::IReg(*arg)))},
            Self::Mul=>{
                res.push(IrOp::Pop(IrVar::IReg(*highest_var)));
                res.push(IrOp::Pop(IrVar::IReg(*highest_var + 1)));
                res.push(IrOp::Mov(IrVar::IReg(*highest_var),IrVar::IReg(*highest_var + 2)));
                res.push(IrOp::Mul(IrVar::IReg(*highest_var + 2),IrVar::IReg(*highest_var + 1)));
                res.push(IrOp::Push(IrVar::IReg(*highest_var + 2)));
                *highest_var+=3;
            },
            Self::Add=>{
                res.push(IrOp::Pop(IrVar::IReg(*highest_var)));
                res.push(IrOp::Pop(IrVar::IReg(*highest_var + 1)));
                res.push(IrOp::Mov(IrVar::IReg(*highest_var),IrVar::IReg(*highest_var + 2)));
                res.push(IrOp::Add(IrVar::IReg(*highest_var + 2),IrVar::IReg(*highest_var + 1)));
                res.push(IrOp::Push(IrVar::IReg(*highest_var + 2)));
                *highest_var+=3;
            },
            Self::Sub=>{
                res.push(IrOp::Pop(IrVar::IReg(*highest_var)));
                res.push(IrOp::Pop(IrVar::IReg(*highest_var + 1)));
                res.push(IrOp::Mov(IrVar::IReg(*highest_var),IrVar::IReg(*highest_var + 2)));
                res.push(IrOp::Sub(IrVar::IReg(*highest_var + 2),IrVar::IReg(*highest_var + 1)));
                res.push(IrOp::Push(IrVar::IReg(*highest_var + 2)));
                *highest_var+=3;
            },
            Self::Ret=>{
                res.push(IrOp::Pop(IrVar::IReg(*highest_var)));
                res.push(IrOp::Return(IrVar::IReg(*highest_var)));
                *highest_var+=1;
            },
            _=>todo!("{self:?}"),
        }
    }
}
#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn method_to_codegen_ir(){
        let method = MethodIR::new(&[ CILOp::LdArg(0),CILOp::LdArg(0),CILOp::Mul,CILOp::LdArg(1),CILOp::LdArg(1),CILOp::Mul,CILOp::Add,CILOp::Ret
        ],2);
        let code = method.ops;
        let expected = [IrOp::Mul(IrVar::IReg(0), IrVar::IReg(0)), IrOp::Mul(IrVar::IReg(1), IrVar::IReg(1)), IrOp::Add(IrVar::IReg(1), IrVar::IReg(0)), IrOp::Return(IrVar::IReg(1))];
        assert!(*(&code as &[IrOp]) == *(&expected as &[IrOp]),"Generated optimized code {code:?} differs from expected code {expected:?}");
    }
    #[test]
    fn method_to_native_ops(){
        let method = MethodIR::new(&[CILOp::LdArg(0),CILOp::LdArg(0),CILOp::Mul,CILOp::LdArg(1),CILOp::LdArg(1),CILOp::Mul,CILOp::Add,CILOp::Ret
        ],2);
        use crate::codegen::CompiledMethod;
        let compiled = CompiledMethod::compile(&method,"Test");
       
    }
}
