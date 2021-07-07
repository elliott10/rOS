global_asm!(include_str!("boot/entry64.asm"));
global_asm!(include_str!("link_user.S"));

use crate::io::getchar;
use crate::sbi;
use crate::consts::*;

use crate::timer::{MMIO_MTIMECMP0, MMIO_MTIMECMP1, MMIO_MTIME};

#[no_mangle]
extern "C" fn rust_main() -> !{

    //crate::interrupt::init_soft();

    crate::interrupt::init(); //未初始化可能会无限重复打印
    crate::timer::init();

    /*
    crate::interrupt::init_soft(); //注意，会影响其他中断??? bug
    sbi::send_ipi(0);
    */

	extern "C" {
		fn _start();
		fn bootstacktop();
	}

    sbi::console_putchar(b'#' as usize);

	sbi::console_putchar_u8(b'X');
	sbi::console_putchar_u8(b'L');
	sbi::console_putchar_u8(b'Y');
	sbi::console_putchar_u8(b'\n');

println!("      ____");
println!(" ____/ ___|___  _ __ ___");
println!("|_  / |   / _ \\| '__/ _ \\");
println!(" / /| |__| (_) | | |  __/");
println!("/___|\\____\\___/|_|  \\___|");
println!();

    /*
    let mut _in: usize = 0;
    let mut _out: usize = 0;
    unsafe {
        llvm_asm!("csrrs $0, time, x0" : "=r"(_out) :::"volatile");
    }
    println!("Now time is:{:#x}", _out);
    */


	println!("_start vaddr = 0x{:x}", _start as usize);
	println!("bootstacktop vaddr = 0x{:x}", bootstacktop as usize);

	println!("--------- Hello World! ---------");

	extern "C" {
		fn end();
	}
	println!("Free physical memory paddr = [{:#x}, {:#x})", end as usize - ( KERNEL_BEGIN_VADDR - KERNEL_BEGIN_PADDR), PHYSICAL_MEMORY_END);

	unsafe{
		llvm_asm!("ebreak"::::"volatile");
	}

    //crate::fs::init();

    //loop {}

    //panic!("end of rust_main()");
	loop {
		if let Some(c) = getchar() {
			match c {
				0x7f | 0x8 => { //0x8 [backspace] ; 而实际qemu运行，[backspace]键输出0x7f, 表示del
                    print!("{} {}", 8 as char, 8 as char );
				},
				10 | 13 => { // 新行或回车
					println!();
				},
				// ANSI ESC序列是多字节：0x1b 0x5b 
                0x1b | 0x5b => {
                    print!("{{{:#x}}}", c);
                },

				//默认
				_ => {
					//print!("{{0x{:x}={}}}", c, c as char);
                    print!("{}", c as char);
				}
			}
		}
	}

}

