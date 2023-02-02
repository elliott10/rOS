use core::arch::global_asm;

global_asm!(include_str!("boot/entry64.asm"));
global_asm!(include_str!("link_user.S"));

use alloc::boxed::Box;

use crate::io::getchar;
use crate::sbi;
use crate::logger;
use crate::consts::*;

use crate::timer::{MMIO_MTIMECMP0, MMIO_MTIMECMP1, MMIO_MTIME};
use core::arch::asm;

use spin::{RwLock, Mutex};
use alloc::vec::Vec;
use alloc::sync::Arc;

use cfg_if::cfg_if;
use lazy_static::lazy_static;
lazy_static! {
    //pub static ref DRIVERS: RwLock<Vec<Arc<Mutex<Driver>>>> = RwLock::new(Vec::new());
    pub static ref DRIVERS: Mutex<Vec<Arc<Mutex<dyn Driver>>>> = Mutex::new(Vec::new());
}

pub trait Driver: Send + Sync {
    fn try_handle_interrupt(&mut self, irq: Option<u32>, opt: usize) -> bool;
}

use buddy_system_allocator::*;
#[global_allocator]
pub static heap: LockedHeap = LockedHeap::new();

#[alloc_error_handler]
pub fn foo(layout: core::alloc::Layout) -> ! {
    println!("alloc_error_handler ! {:?}", layout);
    crate::lang_items::abort()
}

pub struct Provider;
impl super::net::Provider for Provider {
    const PAGE_SIZE: usize = 1 << 12;

    fn alloc_dma(size: usize) -> (usize, usize) {
        //let pages = size / PAGE_SIZE;

        /* 或
           #![feature(new_uninit)]
           let values = Box::<[u32]>::new_zeroed_slice(3);
           let values = unsafe { values.assume_init() };
           */

        //现在只能申请一个页的内存
        let paddr: Box::<[u32]> = Box::new([0; 1024]); // 4096

        let paddr = Box::into_raw(paddr) as *const u32 as usize;
        println!("alloc paddr: {:#x}", paddr);

        let vaddr = paddr;
        (vaddr, paddr)
    }

    fn dealloc_dma(vaddr: usize, size: usize) {
        println!("dealloc_dma unimplemented!");
        /*
           let paddr = virt_to_phys(vaddr);
           for i in 0..size / PAGE_SIZE {
           dealloc_frame(paddr + i * PAGE_SIZE);
           }
           */
    }
}

extern "C" {
    fn end();
}

#[no_mangle]
extern "C" fn rust_main(hartid: usize, dtb: usize) -> !{
    println!("\r\nrust_main, hart id: {}, dtb: {:#x}", hartid, dtb);

    //crate::interrupt::init_soft();

    crate::interrupt::init(); //未初始化可能会无限重复打印
    //crate::timer::init();

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

    use log::{info, debug, trace, warn, error};
    logger::init("DEBUG");

    info!("info");
    debug!("debug");
    warn!("warn");
    error!("error");
    trace!("trace");

    unsafe {
        heap.lock().init(end as usize, PHYSICAL_MEMORY_END - end as usize);
    }

    let mutex = Mutex::new(5);
    {
        let mut num = mutex.lock();
        *num = 6;
    }
    println!("mutex = {:?}", mutex);

    use spin::RwLock;
    let rwlock = RwLock::new(50);
    println!("{:?}", rwlock);
    {
        let r1 = rwlock.read();
        let r2 = rwlock.read();

        println!("{:?}, {:?}", *r1, *r2);
    }
    {
        let mut w = rwlock.write();
        *w += 1;
        println!("{:?}, {:?}", rwlock, *w);
    }

    let x = Box::new(89);
    let v = vec![240, 159, 146, 150];

    println!("Box and vec new: {:?}, {:?}", x, v[1]);

	println!("+++++ Free physical memory paddr = [{:#x}, {:#x})", end as usize - (KERNEL_BEGIN_VADDR - KERNEL_BEGIN_PADDR), PHYSICAL_MEMORY_END);

	unsafe{
		asm!("ebreak");
	}

    //crate::fs::init();

    cfg_if! { if #[cfg(feature = "D1")] {
        //EAPOL packet: 0007326b9940 ca9027e0a80f 888e020000050104000501
        let frame: Box<[u8]> = Box::new(
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x52, 0x54, 0x00, 0x12, 0x34, 0x56,
            0x88, 0x8e, 0x02, 0x00, 0x00, 0x05, 0x01, 0x04, 0x00, 0x05, 0x01]
            );

        let mac: [u8; 6] = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
        let mac: [u8; 6] = [0, 0, 0, 0, 0, 0];
        let mut rtl8211f = crate::net::rtl8211f::RTL8211F::<Provider>::new(&mac);

        unsafe {
            rtl8211f.open();
            rtl8211f.set_rx_mode();
            rtl8211f.adjust_link();

            //开始接收和发送数据
        }

        let mut driver = Arc::new(Mutex::new(rtl8211f));
        DRIVERS.lock().push(driver.clone());

    } else if #[cfg(feature = "fu740")] {

        let mut macb_device = cadence_macb::eth_macb::open().unwrap();

        let ping_frame: Box<[u8]> = Box::new(
                [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x70, 0xb3, 0xd5, 0x92, 0xfa, 0x99, 0x08, 0x06, 0x00, 0x01,
                 0x08, 0x00, 0x06, 0x04, 0x00, 0x01, 0x70, 0xb3, 0xd5, 0x92, 0xfa, 0x99, 0xc0, 0xa8, 0x00, 0x7b, //192.168.0.123
                 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8, 0x00, 0x42]); //ping 192.168.0.66

        // 有时丢失1或2个网络包不能发出？
        cadence_macb::eth_macb_ops::macb_send(&mut macb_device, &ping_frame);
        cadence_macb::eth_macb_ops::macb_send(&mut macb_device, &ping_frame);
        cadence_macb::eth_macb_ops::macb_send(&mut macb_device, &ping_frame);
        cadence_macb::eth_macb_ops::macb_send(&mut macb_device, &ping_frame);

        loop {
            //cadence_macb::eth_macb_ops::macb_send(&mut macb_device, &ping_frame);

            cadence_macb::eth_macb::msdelay(100);

            let mut rx_buf = [0 as u8; 2048];
            cadence_macb::eth_macb_ops::macb_recv(&mut macb_device, &mut rx_buf);
            if rx_buf[0] != 0 {
                print_hex_dump(&rx_buf, 64);
            }
        }

    }
    }

    crate::timer::init();

    println!("OK");

    loop {
        unsafe{ asm!("wfi"); }
    }

    //panic!("end of rust_main()");
    #[allow(unreachable_code)]
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

pub fn print_hex_dump(buf: &[u8], len: usize) {
    //let mut linebuf: [char; 16] = [0 as char; 16];

    use alloc::string::String;
    let mut linebuf = String::with_capacity(32);
    let buf_len = buf.len();

    for i in 0..len {
        if (i % 16) == 0 {
            print!("\t{:?}\nHEX DUMP: ", linebuf);
            //linebuf.fill(0 as char);
            linebuf.clear();
        }

        if i >= buf_len {
            print!(" {:02x}", 0);
        } else {
            print!(" {:02x}", buf[i]);
            //linebuf[i%16] = buf[i] as char;
            linebuf.push(buf[i] as char);
        }
    }
    print!("\t{:?}\n", linebuf);
}
