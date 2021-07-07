	.section .text.entry
	.globl _start
_start:
	#OpenSBI将DTB地址保存在a1寄存器

	#关闭mmu
        csrw satp, zero

	la sp, bootstacktop
	call rust_main

	.section .bss.stack
	.align 12
	.global bootstack
bootstack:
	.space 1024 * 16
	.global bootstacktop
bootstacktop:
