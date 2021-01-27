use riscv::register::{
	sstatus::Sstatus,
	scause::Scause,
};

//表示结构体按照 C 语言标准进行内存布局
#[repr(C)]
pub struct TrapFrame{
	pub x: [usize; 32], //General registers
	pub sstatus: Sstatus,
	pub sepc: usize,
	pub stval: usize,
	pub scause: Scause,
}
