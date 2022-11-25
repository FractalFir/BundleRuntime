#[derive(Debug)]
pub struct ExecutableMemory{
    is_write_protected:bool,
    mem_beg:*mut u8,
    mem_size:usize,
}
#[cfg(target_os = "linux")]
const PROT_READ:i32 = 0x1;
#[cfg(target_os = "linux")]
const PROT_WRITE:i32 = 0x2;
#[cfg(target_os = "linux")]
const PROT_EXEC:i32 = 0x4;
#[cfg(target_os = "linux")]
const MAP_ANONYMUS:i32 = 0x20; 
#[cfg(target_os = "linux")]
const MAP_PRIVATE:i32 = 0x02;
#[cfg(target_os = "linux")]
extern "C" {
    fn mmap(adr:*mut u8,len:usize,prot:i32,flags:i32,fd:i32,offset:usize)->*mut u8;
    fn munmap(adr:*mut u8,size:usize)->i32;
}



const PAGE_SIZE:usize = 4096;
impl ExecutableMemory{
    pub fn new(size:usize)->Self{
        let mem_size = ((size+(PAGE_SIZE - 1))/PAGE_SIZE)*PAGE_SIZE;
        #[cfg(target_os = "linux")]
        unsafe {
            let mem_beg = mmap(0xAAAAAAAAAAAAAAA as *mut u8,4096,PROT_WRITE|PROT_EXEC|PROT_READ,MAP_PRIVATE|MAP_ANONYMUS,0,0);
            assert!(mem_beg as usize != 0);
            Self{mem_size,mem_beg,is_write_protected:false}
        }
        #[cfg(not(target_os = "linux"))]
        compile_error!("Systems other than Linux are not supported!"); 
    }
}
impl<'a> ExecutableMemory{
    pub fn get_slice_at(&'a self,beg:usize,size:usize)->&'a [u8]{
        if beg+size>=self.mem_size {panic!("Index outside mapped protected executable memory!")}
        else {unsafe{std::slice::from_raw_parts((self.mem_beg as usize + beg) as *mut u8,size)}}
    }
    pub fn get_mut_slice_at(&'a self,beg:usize,size:usize)->&'a mut [u8]{
        if beg+size>=self.mem_size {panic!("Index outside mapped protected executable memory!")}
        else {unsafe{std::slice::from_raw_parts_mut((self.mem_beg as usize + beg) as *mut u8,size)}}
    }
    pub fn get_ptr(&self,offset:usize)->usize{
         if offset>=self.mem_size  {panic!("Index outside mapped protected executable memory!")}
         else {
            (self.mem_beg as usize) + offset
         }
    }
}
impl Drop for ExecutableMemory{
    fn drop(&mut self){
        unsafe{munmap(self.mem_beg,self.mem_size)};
    }
} 
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alloc_exec_mem(){
        let exec_mem = ExecutableMemory::new(4096);
    }
    #[test]
    fn read_exec_mem(){
        let exec_mem = ExecutableMemory::new(4096);
        let slice = exec_mem.get_slice_at(100,150);
        println!("slice:{slice:?}");
        let val = slice[0];
        assert!(val == 0);
    }
    #[test]
    fn write_exec_mem(){
        let exec_mem = ExecutableMemory::new(4096);
        let slice = exec_mem.get_mut_slice_at(100,150);
        println!("slice:{slice:?}");
        slice[0] = 0xFF;
    }
    #[test]
    fn execute_exec_mem(){
        let exec_mem = ExecutableMemory::new(4096);
        let mut slice = exec_mem.get_mut_slice_at(0,6);
        println!("slice:{slice:?}");
        let mut i:usize = 0;
        slice[0] = 0o303;
        slice[1] = 0o125;
        let stub_2_ptr:(extern "C" fn()) = unsafe{std::mem::transmute(slice.as_ptr())};
        println!("stub_2_ptr:{:x}",stub_2_ptr as usize);
        unsafe{(stub_2_ptr)()};
        drop(exec_mem);
    }
}
