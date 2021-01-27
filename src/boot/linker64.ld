OUTPUT_ARCH(riscv)
ENTRY(_start)

/*
BASE_ADDRESS = 0x80200000;
BASE_ADDRESS = 0x80010000;

BASE_ADDRESS 需对接上opensbi的jump地址, 如果地址太小如0x8000c000，或设置不对, 都可能出现无限重复打印的情况;
也有可能中断未初始化，然后却调用ebreak
也可以尝试： make clean
*/

BASE_ADDRESS = 0x80010000;

/* Qemu
BASE_ADDRESS = 0x80200000;
*/

BASE_ADDRESS = 0x80200000;

SECTIONS
{
    /* Load the kernel at this address: "." means the current address */
    . = BASE_ADDRESS;
    start = .;

    .text : {
        stext = .;
        *(.text.entry)
        *(.text .text.*)
        . = ALIGN(4K);
        etext = .;
    }

    .rodata : {
        srodata = .;
        *(.rodata .rodata.*)
        . = ALIGN(4K);
        erodata = .;
    }

    .data : {
        sdata = .;
        *(.data .data.*)
        edata = .;
    }

    .stack : {
        *(.bss.stack)
    }

    .bss : {
        sbss = .;
        *(.bss .bss.*)
        ebss = .;
    }

    PROVIDE(end = .);
}
