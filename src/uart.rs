use core::convert::TryInto;
use core::fmt::{Error, Write};

//use crate::console::push_stdin;

pub struct Uart {
	base_address: usize,
}

// 结构体Uart的实现块
impl Uart {
	pub fn new(base_address: usize) -> Self {
		Uart {
			base_address
		}
	}
/*
uart初始化
设置字长为8-bits (LCR[1:0])
使能先进先出FIFOs (FCR[0])
使能接受中断(IER[0]), 在这只使用轮询的方式而不用中断

*/
pub fn init(&mut self) {
    let ptr = self.base_address as *mut u32;
	unsafe {
        /* 记得在UART初始化之前，一定先把riscv外部中断关闭 */

        // 等待UART为非busy状态
        //while (ptr.add(31).read_volatile() & 0x1) != 0 {/* USR */}

        // 关中断
		ptr.add(1).write_volatile(0);

		// FCR at offset 2
		ptr.add(2).write_volatile(0x1);

        // Halt TX
		ptr.add(41).write_volatile(0x03);

		// LCR at base_address + 3, DLAB = 0
		// 置位:    bit 1      bit 0
		let lcr = (1 << 1) | (1 << 0);

		// 设置波特率，除子，取整等
		// 2.729 MHz (22,729,000 cycles per second) --> 波特率 2400 (BAUD)
        // D1 = 24 MHz

		// 根据NS16550a规格说明书计算出divisor
		// divisor = ceil( (clock_hz) / (baud_sps x 16) )
		// divisor = ceil( 22_729_000 / (2400 x 16) ) = ceil( 591.901 ) = 592

		// divisor寄存器是16 bits
		let divisor: u16 = 13; //115200
		let divisor_least: u32 = (divisor as u32 & 0xff);
		let divisor_most:  u32 = (divisor as u32 >> 8);

		// DLL和DLM会与其它寄存器共用基地址，需要设置DLAB来切换选择寄存器
		// LCR base_address + 3, DLAB = 1
        // 注意，D1板子在busy状态时写LCR，会触发busy中断, 要读USR寄存器来清；
		ptr.add(3).write_volatile(lcr | (1 << 7));

		//写DLL和DLM来设置波特率, 把频率22.729 MHz的时钟划分为每秒2400个信号
		ptr.add(0).write_volatile(divisor_least);
		ptr.add(1).write_volatile(divisor_most);

        //Halt update and wait
		ptr.add(41).write_volatile(0x4 | 0x2 | 0x1);
        while ptr.add(41).read_volatile() & 0x4 != 0 {}

		// 设置后, 清空DLAB
		ptr.add(3).write_volatile(lcr);

        //Halt reset
		ptr.add(41).write_volatile(0);

        // Fifo reset
        ptr.add(2).write_volatile(0x7);
        ptr.add(4).write_volatile(0x3);

		//IER at offset 1, 开中断
		ptr.add(1).write_volatile(1);
	}
    println!("\nUART OK");
}

pub fn d1pac_init(&mut self) {
    println!("--- d1 pac init start ---");

    use d1_pac::uart::RegisterBlock;
    let regb = self.base_address as *mut RegisterBlock;
    let regb = unsafe { regb.as_ref().unwrap() };

    // 记得先把plic 外部中断关了
    //
    //等待释放fifo缓存
    //while regb.usr.read().busy().is_busy() {/* USR */}

    regb.ier().reset();
    regb.fcr().write(|w| w.fifoe().set_bit());

    regb.halt.write(|w| w.halt_tx().enabled()
                    .chcfg_at_busy().enable());

    regb.lcr.write(|w| w.dls().eight()
                   .dlab().divisor_latch());
    regb.dll().write(|w| w.dll().variant(26 & 0xff)); //115200
    regb.dlh().write(|w| w.dlh().variant(0));

    regb.halt.write(|w| w.change_update().update_trigger());
    while !regb.halt.read().change_update().is_finished() {}

    regb.lcr.write(|w| w.dls().eight()
                   .dlab().rx_buffer());
    regb.halt.reset();

    regb.fcr().write(|w| w.fifoe().set_bit());
    regb.mcr.write(|w| w.dtr().asserted().rts().asserted());

    regb.ier().write(|w| w.erbfi().enable());

    println!("--- d1 pac init end ---");
}

pub fn simple_init(&mut self) {
	//let ptr = self.base_address as *mut u8;
	let ptr = self.base_address as *mut u32;
	unsafe {
        //D1 ALLWINNER的uart中断使能

        // Enable FIFO; (base + 2)
        ptr.add(2).write_volatile(0x7);

        // MODEM Ctrl; (base + 4)
        ptr.add(4).write_volatile(0x3);

        // D1 UART_IER offset = 0x4
        //
        // Enable interrupts; (base + 1)
        ptr.add(1).write_volatile(0x1);
    }
}

pub fn get(&mut self) -> Option<u8> {
	let ptr = self.base_address as *mut u32;
	unsafe {
		//查看LSR的DR位为1则有数据
        //if ptr.add(5).read_volatile() & 0b1 == 0 {
		if ptr.add(31).read_volatile() & 0b1000 == 0 {
			None
		} else {
			Some((ptr.add(0).read_volatile() & 0xff) as u8)
		}
	}

}

pub fn put(&mut self, c: u8) {
	let ptr = self.base_address as *mut u8;
	unsafe {
        //if USR TX fifo is full
        while ((ptr as *const u32).add(31).read_volatile() & 0b10) == 0 {}

		//此时transmitter empty, THR有效位是8
		ptr.add(0).write_volatile(c);
	}
}

}

// 需要实现的write_str()重要函数
impl Write for Uart {
	fn write_str(&mut self, out: &str) -> Result<(), Error> {
        for c in out.bytes(){
            if c == b'\n' {
                self.put(b'\r');
            }
            self.put(c);
        }
		Ok(())
	}
}

/*
fn unsafe mmio_write(address: usize, offset: usize, value: u8) {
	//write_volatile() 是 *mut raw 的成员；
	//new_pointer = old_pointer + sizeof(pointer_type) * offset
	//也可使用reg.offset

	let reg = address as *mut u8;
	reg.add(offset).write_volatile(value);
}

fn unsafe mmio_read(address: usize, offset: usize, value: u8) -> u8 {

	let reg = address as *mut u8;

	//读取8 bits
	reg.add(offset).read_volatile(value) //无分号可直接返回值
}
*/

pub fn handle_interrupt() {
    // D1 ALLWINNER
    let mut my_uart = Uart::new(0x02500000);

	if let Some(c) = my_uart.get() {
		//CONSOLE
		//push_stdin(c);

        let c = c & 0xff;

		match c {
			0x7f => { //0x8 [backspace] ; 而实际qemu运行，[backspace]键输出0x7f, 表示del
				print!("{} {}", 8 as char, 8 as char);
			},
			10 | 13 => { // 新行或回车
				println!();
			},
			_ => {
				print!("{}", c as char);
			},
		}
	}
}

