#[derive(Debug,PartialEq,Clone,Copy)]
pub enum IrVar{
    Arg(u8),
    Tmp(u8),
}
impl IrVar{
    fn alias(&mut self,var:IrVar,alias:IrVar){
        (*self) = if (*self) == var {alias} else {*self};
    }
}
impl Eq for IrVar{}
#[derive(Debug,Clone,Copy,PartialEq)]
pub enum IrOp{
    Add(IrVar,IrVar,IrVar),
    Mul(IrVar,IrVar,IrVar),
    Pop(IrVar),
    Push(IrVar),
    Return(IrVar),
    Nop,
}
impl Eq for IrOp{}
impl IrOp{
    fn alias_variable(&mut self,var:IrVar,alias:IrVar){
        match self{
            IrOp::Add(ref mut a,ref mut b,ref mut dst)=>{
                a.alias(var,alias);
                b.alias(var,alias);
                dst.alias(var,alias);
            },
            IrOp::Mul(ref mut a,ref mut b,ref mut dst)=>{
                a.alias(var,alias);
                b.alias(var,alias);
                dst.alias(var,alias);
            },
            IrOp::Push(ref mut src)=>src.alias(var,alias),
            IrOp::Pop(ref mut dst)=>dst.alias(var,alias),
            IrOp::Return(ref mut ret)=>ret.alias(var,alias),
            IrOp::Nop=>(), //No variables, ignore
            _=>todo!("{self:?} variant is not supported by varaible aliaser!"),
        }
    }
    fn is_meanigfull(&self)->bool{
        match self{
            IrOp::Nop=>false,
            _=>true,
        }
    }
}
fn alias_variable(ops:&mut [IrOp],var:IrVar,alias:IrVar){
    for mut op in ops{
        op.alias_variable(var,alias);
    }
}
// Tmp is used as destination and swapped with ops. This saves allocations.
fn push_pop_inline(ops:&mut Vec<IrOp>){
   let mut aliases:Vec<(IrVar,IrVar,usize,usize)> = Vec::new(); //add push & pop loc
   let mut op_iter = ops.iter();
   let mut index = 0;
   while let Some(op) = op_iter.next(){
        match op{
            IrOp::Push(src)=>{
                let mut src = *src;
                let mut src_index = index;
                index+=1;
                while let Some(op) = op_iter.next(){
                        match op{
                            IrOp::Pop(dst)=>{
                               aliases.push((*dst,src,src_index,index)); 
                               break;
                            },
                            IrOp::Push(new_src)=>{src = *new_src;src_index = index},
                            //TODO when labels:If label, break.
                            _=>(),
                        };
                        index+=1;
                }
            },
            _=>(),
        };
        index+=1;
   }
   // enable for debug if this function fails after a change to check if aliasing works
   /*
   for alias in &aliases{
        println!("als: {:?} is now {:?} and {:?} and {:?} are gone.",alias.0,alias.1,ops[alias.2],ops[alias.3]); 
   }
   */
   for alias in &aliases{
        alias_variable(ops,alias.0,alias.1);
        ops[alias.2] = IrOp::Nop;
        ops[alias.3] = IrOp::Nop;
   } 
   ops.retain(IrOp::is_meanigfull)
}
fn optimize_ir(ops:&mut Vec<IrOp>){
    let mut last_size = ops.len();
    'opt: loop {
        push_pop_inline(ops);
        let curr_size = ops.len();
        if !(curr_size < last_size){return};
        last_size = curr_size;
    }
}
#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_opt_stack_simple_ir(){
        let arg0 = IrVar::Arg(0);
        let arg1 = IrVar::Arg(1);
        let tmp0 = IrVar::Tmp(0);
        let tmp1 = IrVar::Tmp(1);
        let tmp2 = IrVar::Tmp(2);
        let tmp3 = IrVar::Tmp(3);
        let tmp4 = IrVar::Tmp(4);
        let tmp5 = IrVar::Tmp(5);
        let tmp6 = IrVar::Tmp(6);
        let tmp7 = IrVar::Tmp(7);
        let tmp8 = IrVar::Tmp(8);
        let ret = IrVar::Tmp(9);
        let mut code = vec![
            IrOp::Push(arg0),
            IrOp::Push(arg0),
            IrOp::Pop(tmp0),IrOp::Pop(tmp1),IrOp::Add(tmp0,tmp1,tmp2),IrOp::Push(tmp2),
            IrOp::Push(arg1),
            IrOp::Push(arg1),
            IrOp::Pop(tmp3),IrOp::Pop(tmp4),IrOp::Add(tmp3,tmp4,tmp5),IrOp::Push(tmp5),
            IrOp::Pop(tmp6),IrOp::Pop(tmp7),IrOp::Mul(tmp6,tmp7,tmp8),IrOp::Push(tmp8),
            IrOp::Pop(ret),IrOp::Return(ret)
            ];
        println!("{}:\n{code:?}\n",code.len());
        //alias_variable(&mut code,tmp0,arg0);
        optimize_ir(&mut code);
        let expected = [IrOp::Add(IrVar::Arg(0), IrVar::Arg(0), IrVar::Tmp(2)), IrOp::Add(IrVar::Arg(1), IrVar::Arg(1), IrVar::Tmp(5)), IrOp::Mul(IrVar::Tmp(5), IrVar::Tmp(2), IrVar::Tmp(8)), IrOp::Return(IrVar::Tmp(8))];
        assert!(*(&code as &[IrOp]) == *(&expected as &[IrOp]),"Generated optimized code {code:?} differs from expected code {expected:?}");
    }
}
