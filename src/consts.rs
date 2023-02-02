
/*
 * 而x86访问其他外设, x86 单独提供了in, out 指令来访问不同于内存的IO地址空间;
 * 很多 CPU（如 RISC-V，ARM，MIPS 等）通过 MMIO(Memory Mapped I/O) 技术将外设映射到一段物理地址，这样访问其他外设就和访问物理内存一样啦
 */
// paddr + offset = vaddr

cfg_if::cfg_if! {
    if #[cfg(feature = "D1")] {
        // D1 ALLWINNER
        // OR kernel paddr = 0x45000000
        pub const PHYSICAL_MEMORY_END: usize = 0x50020000; // 256M
        pub const KERNEL_BEGIN_PADDR: usize = 0x40020000;
        pub const KERNEL_BEGIN_VADDR: usize = 0x40020000;

        pub const PLIC_BASE: usize = 0x1000_0000;
        pub const UART_BASE: usize = 0x0250_0000;

    } else if #[cfg(feature = "fu740")] {
        pub const PHYSICAL_MEMORY_END: usize = 0x90000000;
        pub const KERNEL_BEGIN_PADDR: usize = 0x80200000;
        pub const KERNEL_BEGIN_VADDR: usize = 0x80200000;

        pub const PLIC_BASE: usize = 0x0c00_0000;
        pub const UART_BASE: usize = 0x1001_0000;
    } else {
        // DRAM 默认128MB, 可见qemu/hw/riscv/virt.c
        pub const PHYSICAL_MEMORY_END: usize = 0x88000000;
        // 物理地址前面被opensbi占用
        pub const KERNEL_BEGIN_PADDR: usize = 0x80200000;
        pub const KERNEL_BEGIN_VADDR: usize = 0x80200000;

        pub const PLIC_BASE: usize = 0x0c00_0000;
        pub const UART_BASE: usize = 0x1000_0000;
    }
}
