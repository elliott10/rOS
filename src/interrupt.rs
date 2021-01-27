use riscv::register::{
	scause::{
		self,
		Trap,
		Exception,
		Interrupt,
	},
    sie,
	sepc,
	stvec,
	sscratch,
	sstatus,
};
use crate::timer::{
	TICKS,
	clock_set_next_event,
    clock_close,
};
use crate::context::TrapFrame;
use crate::sbi;
use crate::plic;
use crate::uart;

use k210_hal::{clock::Clocks, pac, prelude::*};
use k210_hal::serial::SerialExt;



global_asm!(include_str!("trap/trap.asm"));

fn init_m(){
    use riscv::register::{mhartid, mstatus, mie};

    /*
    use k210_hal::plic::Priority;
    use k210_hal::pac::Interrupt;
    use k210_hal::gpiohs::Edge;

    let hartid = mhartid::read();
    */

    unsafe{
        /*
        pac::PLIC::set_priority(Interrupt::GPIOHS0, Priority::P7);


        pac::PLIC::set_threshold(mhartid::read(), Priority::P0);
        pac::PLIC::unmask(mhartid::read(), Interrupt::GPIOHS0);
        */

		mstatus::set_mie(); //如果在S态下设置，会出现非法指令错误
        mie::set_mext();
    }
}

pub fn init(){
	unsafe{
		extern "C" {
			fn __alltraps();
		}

		sscratch::write(0);
		stvec::write(__alltraps as usize, stvec::TrapMode::Direct);

		sstatus::set_sie();
        //当硬件决定触发时钟中断时，会将sip寄存器的STIP位设置为1;
        //当一条指令执行完毕后，如果发现STIP为1，此时如果sie 的STIE位也为1，会进入S态时钟中断的处理程序

        //M模式时的初始化
        //init_m();

        //注意！bug! 如果之后sie::set_ssoft(),会出现无法收到S态的外部中断和时钟中断
        //打开外部中断
        sie::set_sext();
        init_ext();

        init_uart();
	}
	println!("+++ setup interrupte! +++");
}
/*
 PMP, 物理内存保护, 允许M模式指定U模式可以访问的内存地址;

 mideleg, 机器中断委派；
 medeleg, 机器中断委派;
*/

#[no_mangle]
pub fn rust_trap(tf: &mut TrapFrame){
    let sepc = tf.sepc;
    let stval = tf.stval;
    let is_int = tf.scause.bits() >> 63;
    let code = tf.scause.bits() & !(1 << 63);

	match tf.scause.cause() {
		Trap::Exception(Exception::Breakpoint) => breakpoint(&mut tf.sepc),
		Trap::Exception(Exception::IllegalInstruction) => panic!("IllegalInstruction: {:#x}->{:#x}", sepc, stval),
        Trap::Exception(Exception::LoadFault) => panic!("Load access fault: {:#x}->{:#x}", sepc, stval),
        Trap::Exception(Exception::StoreFault) => panic!("Store access fault: {:#x}->{:#x}", sepc, stval),
        Trap::Exception(Exception::LoadPageFault) => page_fault(stval, tf),
        Trap::Exception(Exception::StorePageFault) => page_fault(stval, tf),
        Trap::Exception(Exception::InstructionPageFault) => page_fault(stval, tf),
		Trap::Interrupt(Interrupt::SupervisorTimer) => super_timer(),
		Trap::Interrupt(Interrupt::SupervisorSoft) => super_soft(),
		Trap::Interrupt(Interrupt::SupervisorExternal) => plic::handle_interrupt(),
		_ => panic!("Undefined Trap: {:#x} {:#x}", is_int, code)
	}
}
/*
特殊的k210

非法指令：
80012218:   c0102573                rdtime  a0

且无法通过tval取得指令值;

*/

fn breakpoint(sepc: &mut usize){
	println!("A breakpoint set @0x{:x} ", sepc);

	//sepc为触发中断指令ebreak的地址
	//防止无限循环中断，让sret返回时跳转到sepc的下一条指令地址
	*sepc +=2
}

fn page_fault(stval: usize, tf: &mut TrapFrame){
    panic!("EXCEPTION: Page Fault @ {:#x}->{:#x}", tf.sepc, stval);
}

fn super_timer(){
    clock_set_next_event();
	unsafe {
		//多个线程都能访问，同时可能会造成错误
		TICKS += 1;
		if (TICKS == 100){
			TICKS = 0;
			println!("100 ticks");
		}
	}

	//发生外界中断时，epc的指令还没有执行，故无需修改epc到下一条
}

fn init_uart(){
    uart::Uart::new(0x1000_0000).init();

    use core::fmt::Write;
    write!(crate::uart::Uart::new(0x1000_0000), "Uart writing test !\n");

    /*
    let p = pac::Peripherals::take().unwrap();
    let clocks = Clocks::new();
    let serial = p.UARTHS.configure(115_200.bps(), &clocks);
    let (tx, rx) = serial.split();
    */

    // k210 0x38000000
    // Interrupt Enable Register: 0x10, 32位寄存器, 使能接收中断：bit1 = 1 <= (1 << 1)
    /*
    let mut ie: u32 = 0;
    unsafe {
        ie = ((0x38000000 + 0x10) as *mut u32).read_volatile();
        ((0x38000000 + 0x10) as *mut u32).write_volatile( ie | (1 << 1));


    println!("0x38000000 : ie: {:#x} -> {:#x}", ie, ((0x38000000 + 0x10) as *mut u32).read_volatile());
    }
    */

    println!("+++ Setting up UART interrupts +++");
}

pub fn init_ext(){
    // Qemu virt
    // UART0 = 10
    plic::set_priority(10, 7);
    plic::set_threshold(0);
    plic::enable(10);

    // k210 UART = 33
    /*
    plic::set_priority(33, 7);
    plic::set_threshold(0);
    plic::enable(33);


    // set opensbi s_insn()
    sbi::set_s_insn(s_insn as usize);
    */
    
    println!("+++ Setting up PLIC +++");
}

fn super_soft(){
    sbi::clear_ipi();
    println!("Interrupt::SupervisorSoft!");
}

pub fn init_soft(){
    unsafe {
        sie::set_ssoft();
    }
	println!("+++ setup soft int! +++");
}

pub fn s_insn(){
    println!("+++ s_insn()");

}

