use crate::sbi::set_timer;
use riscv::register::{
	time,
	sie
};

pub const MMIO_MTIMECMP0: *mut u64 = 0x0200_4000usize as *mut u64;
pub const MMIO_MTIMECMP1: *mut u64 = 0x0200_4008usize as *mut u64;
pub const MMIO_MTIME: *const u64 = 0x0200_BFF8 as *const u64;

pub static mut TICKS: usize = 0;

//static TIMEBASE: u64 = 100000;
static TIMEBASE: u64 = 10_000_000;

pub fn init(){
	unsafe {
		TICKS = 0;
		sie::set_stimer(); //这里时钟中断，先触发M态的IRQ_M_TIMER，然后被opensbi收到后, 转发到S态的IRQ_S_TIMER；

        // 当在M态不进行时钟中断委派到S态时，Qemu的M态可接收到S态时钟中断
        // K210的M态无法收到S态中断，时钟中断和软件中断可以委派到S态来收, 而PLIC外部中断即使委派了也不行, 真是大坑! 
	}

	clock_set_next_event();
	println!("+++ setup timer! +++");
}

pub fn clock_set_next_event(){
    set_timer( time::read() as u64 + TIMEBASE);

	println!("S timer!");
    
    //k210 可能需要页表，才可读写该地址
    /*
    unsafe {
        //MMIO_MTIMECMP.write_volatile(MMIO_MTIME.read_volatile() + 10_000_000);
        set_timer( MMIO_MTIME.read_volatile() + TIMEBASE );
    }
    */

}

pub fn clock_close(){
    unsafe {
        sie::clear_stimer();
    }
}

