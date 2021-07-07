use crate::uart;

//通过MMIO地址对平台级中断控制器PLIC的寄存器进行设置
//
//Source 1 priority: 0x0c000004
//Source 2 priority: 0x0c000008
const PLIC_PRIORITY:   usize = 0x1000_0000;
//Pending 32位寄存器，每一位标记一个中断源ID
const PLIC_PENDING:    usize = 0x1000_1000;

//Target 0 threshold: 0x0c200000
//Target 0 claim    : 0x0c200004
//
//Target 1 threshold: 0x0c201000 *
//Target 1 claim    : 0x0c201004 *

const PLIC_THRESHOLD:  usize = 0x1020_1000;
const PLIC_CLAIM:      usize = 0x1020_1004;

//注意一个核的不同权限模式是不同Target
//Target: 0  1  2        3  4  5
// Hart0: M  S  U Hart1: M  S  U
//
//target 0 enable: 0x0c002000
//target 1 enable: 0x0c002080 *
const PLIC_INT_ENABLE: usize = 0x1000_2080 ; //基于opensbi后一般运行于Hart0 S态，故为Target1

//PLIC是async cause 11
//声明claim会清除中断源上的相应pending位。
//即使mip寄存器的MEIP位没有置位, 也可以claim; 声明不被阀值寄存器的设置影响；
//获取按优先级排序后的下一个可用的中断ID
pub fn next() -> Option<u32> {
	let claim_reg = PLIC_CLAIM as *const u32;
	let claim_no;
	unsafe {
		claim_no = claim_reg.read_volatile();
        //println!("PLIC claim, Read {:#x}, Value: {:#x}", claim_reg as usize, claim_no as usize);
	}
	if claim_no == 0 {
		None //没有可用中断待定
	}else{
		Some(claim_no)
	}
}

//claim时，PLIC不再从该相同设备监听中断
//写claim寄存器，告诉PLIC处理完成该中断
// id 应该来源于next()函数
pub fn complete(id: u32) {
	let complete_reg = PLIC_CLAIM as *mut u32; //和claim相同寄存器,只是读或写的区别
	unsafe {
		complete_reg.write_volatile(id);
	}
}

//看的中断ID是否pending
pub fn is_pending(id: u32) -> bool {
	let pend = PLIC_PENDING as *const u32;
	let actual_id = 1 << id;
	let pend_ids;
	unsafe {
		pend_ids = pend.read_volatile();
        println!("PLIC pending: Read {:#x}, Value: {:#x}", pend as usize, pend_ids as usize);
	}
	actual_id & pend_ids != 0
}

//使能target中某个给定ID的中断
//中断ID可查找qemu/include/hw/riscv/virt.h, 如：UART0_IRQ = 10
pub fn enable(id: u32) {
	let enables = PLIC_INT_ENABLE as *mut u32; //32位的寄存器

    let actual_id = 1 << id;
    unsafe {
        enables.write_volatile(enables.read_volatile() | actual_id);
        // 0x0c00_2000 <=~ (1 << 10)

        let read_enable = enables.read_volatile();
        println!("PLIC enable, Read {:#x}, Value: {:#x}", enables as usize, read_enable as usize);
    }
}

//设置中断源的优先级，分0～7级，7是最高级, eg:这里id=10, 表示第10个中断源的设置, prio=1
pub fn set_priority(id: u32, prio: u8) {
	//let actual_prio = prio as u32 & 7;
	let actual_prio = prio as u32 & 0x1f;
	let prio_reg = PLIC_PRIORITY as *mut u32;
	unsafe {
		prio_reg.add(id as usize).write_volatile(actual_prio); //0x0c000000 + 4 * 10 <= 1 = 1 & 7

        let read_prio = prio_reg.add(id as usize).read_volatile();
        println!("PLIC priority, Read {:#x}, Value: {:#x}", prio_reg.add(id as usize) as usize, read_prio as usize);
	}
}

//设置中断target的全局阀值［0..7]， <= threshold会被屏蔽
pub fn set_threshold(tsh: u8) {
	let actual_tsh = tsh & 0x1f; //使用0b11111保留最后5位
	let tsh_reg = PLIC_THRESHOLD as *mut u32;
	unsafe {
		tsh_reg.write_volatile(actual_tsh as u32); // 0x0c20_0000 <= 0 = 0 & 7

        let read_tsh = tsh_reg.read_volatile();
        println!("PLIC threshold, Read {:#x}, Value: {:#x}", tsh_reg as usize, read_tsh as usize);
	}
}

pub fn handle_interrupt() {
	if let Some(interrupt) = next() {
		match interrupt {
			1..=8 => {
				//virtio::handle_interrupt(interrupt);
			},
			//10 => { //UART中断ID是10
			18 => { //D1 ALLWINNER
			//33 => { //UART中断ID是10

				//println!("External interrupt: {}", interrupt);

        use crate::sbi;
        let sbic = sbi::console_getchar();
        print!("sbi: {}", sbic);

                //uart::handle_interrupt();
			},
			_ => {
				println!("Unknown external interrupt: {}", interrupt);
			},
		}

		//这将复位pending的中断，允许UART再次中断。
		//否则，UART将被“卡住”
		complete(interrupt);
	}
}


