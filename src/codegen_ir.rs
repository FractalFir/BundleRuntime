/// TODO: rethink the use of word "alias" in code(it is not accurate depiction of what some function do and may be misleading).
#[derive(Debug,PartialEq,Clone,Copy)]
pub(crate) enum IrVar{
//   Arg(u8),
    IReg(usize),
}
impl IrVar{
    fn alias(&mut self,var:IrVar,alias:IrVar){
        (*self) = if (*self) == var {alias} else {*self};
    }
}
impl Eq for IrVar{}
#[derive(Debug,Clone,Copy,PartialEq)]
pub(crate) enum IrOp{
    Add(IrVar,IrVar),
    Mul(IrVar,IrVar),
    Mov(IrVar,IrVar),
    Pop(IrVar),
    Push(IrVar),
    Return(IrVar),
    Nop,
}
impl Eq for IrOp{}
impl IrOp{
    /// Replaces internal references to var with an alias.
    fn alias_variable(&mut self,var:IrVar,alias:IrVar){
        match self{
            IrOp::Add(ref mut val,ref mut addend)=>{
                val.alias(var,alias);
                addend.alias(var,alias);
            },
            IrOp::Mul(ref mut multiplicand,ref mut multiplier)=>{
                multiplicand.alias(var,alias);
                multiplier.alias(var,alias);
            },
            IrOp::Mov(ref mut source,ref mut target)=>{
                source.alias(var,alias);
                target.alias(var,alias);
            },
            IrOp::Push(ref mut src)=>src.alias(var,alias),
            IrOp::Pop(ref mut dst)=>dst.alias(var,alias),
            IrOp::Return(ref mut ret)=>ret.alias(var,alias),
            IrOp::Nop=>(), //No variables, ignore
            _=>todo!("{self:?} variant is not supported by variable aliaser!"),
        }
    }
    /// Checks if a variable is used by this instruction
    fn uses_variable(&self,var:IrVar)->bool{
        match self{
            IrOp::Add(val,addend)=>*val == var || *addend == var,
            IrOp::Mul(multiplicand,multiplier)=>*multiplicand == var || *multiplier == var,
            IrOp::Mov(source,target)=>*source == var || *target == var,
            IrOp::Push(src)=>*src == var,
            IrOp::Pop(dst)=>*dst == var,
            IrOp::Return(ret)=>*ret == var,
            IrOp::Nop=>false, //No variables, ignore
        }
    }
    /// Checks whether this instruction does anything (is not equivalent to a Nop). 
    fn is_meanigfull(&self)->bool{
        match self{
            IrOp::Nop=>false,
            IrOp::Mov(src,target)=>src != target,
            _=>true,
        }
    }
}
/// Replaces one variable with another.
fn alias_variable(ops:&mut [IrOp],var:IrVar,alias:IrVar){
    for mut op in ops{
        op.alias_variable(var,alias);
    }
}
fn is_var_used_after(ops:&Vec<IrOp>,var:IrVar,pos:usize)->bool{
    let mut iter = ops.iter().skip(pos + 1);
    while let Some(op) = iter.next(){
        if op.uses_variable(var){return true};
    }
    return false;
}
// IReg is used as destination and swapped with ops. This saves allocations.
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
}
fn unneded_move_inline(ops:&mut Vec<IrOp>){
   let mut aliases:Vec<(IrVar,IrVar)> = Vec::new(); //add push & pop loc
   let mut op_iter = ops.iter();
   let mut index = 0;
   while let Some(op) = op_iter.next(){
        match op{
            IrOp::Mov(src,dst)=>{
                if !is_var_used_after(ops,*src,index){
                    aliases.push((*src,*dst));
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
        //reverse order of tuple because source (alias.0) becomes overrides target (alias.1)
        alias_variable(ops,alias.1,alias.0);
   } 
}
/*
fn sqish_vars(ops:&mut Vec<IrOp>){
    let vars = Vec::new();
    for op in ops{
        
    }
}*/
pub(crate) fn optimize_ir(ops:&mut Vec<IrOp>){
    let mut last_size = ops.len();
    'opt: loop {
        push_pop_inline(ops);
        unneded_move_inline(ops);
        ops.retain(IrOp::is_meanigfull);
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
        let arg0 = IrVar::IReg(0);
        let arg1 = IrVar::IReg(1);
        let tmp0 = IrVar::IReg(2);
        let tmp1 = IrVar::IReg(3);
        let tmp2 = IrVar::IReg(4);
        let tmp3 = IrVar::IReg(5);
        let tmp4 = IrVar::IReg(6);
        let tmp5 = IrVar::IReg(7);
        let tmp6 = IrVar::IReg(8);
        let tmp7 = IrVar::IReg(9);
        let tmp8 = IrVar::IReg(10);
        let ret = IrVar::IReg(11);
        let mut code = vec![
            IrOp::Push(arg0),
            IrOp::Push(arg0),
            IrOp::Pop(tmp0),IrOp::Pop(tmp1),IrOp::Mov(tmp0,tmp2),IrOp::Mul(tmp2,tmp1),IrOp::Push(tmp2),
            IrOp::Push(arg1),
            IrOp::Push(arg1),
            IrOp::Pop(tmp3),IrOp::Pop(tmp4),IrOp::Mov(tmp3,tmp5),IrOp::Mul(tmp5,tmp4),IrOp::Push(tmp5),
            IrOp::Pop(tmp6),IrOp::Pop(tmp7),IrOp::Mov(tmp6,tmp8),IrOp::Add(tmp8,tmp7),IrOp::Push(tmp8),
            IrOp::Pop(ret),IrOp::Return(ret)
            ];
        println!("{}:\n{code:?}\n",code.len());
        //alias_variable(&mut code,tmp0,arg0);
        optimize_ir(&mut code);
        println!("{}:\n{code:?}\n",code.len());
        let expected = [IrOp::Mul(IrVar::IReg(0), IrVar::IReg(0)), IrOp::Mul(IrVar::IReg(1), IrVar::IReg(1)), IrOp::Add(IrVar::IReg(1), IrVar::IReg(0)), IrOp::Return(IrVar::IReg(1))];
        assert!(*(&code as &[IrOp]) == *(&expected as &[IrOp]),"Generated optimized code {code:?} differs from expected code {expected:?}");
    }
}
