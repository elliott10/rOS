target := riscv64imac-unknown-none-elf
mode := debug
kernel := target/$(target)/$(mode)/os
kernel_debuginfo := target/$(target)/$(mode)/os.debuginfo
bin := target/$(target)/$(mode)/kernel.bin

objdump := rust-objdump --arch-name=riscv64
objcopy := rust-objcopy --binary-architecture=riscv64

out_dir := rootfs/

simplefs_img := riscv64.img

.PHONY: kernel build clean qemu run env

env:
	cargo install cargo-binutils
	rustup component add llvm-tools-preview rustfmt
	rustup target add $(target)

export USER_IMG = riscv64.img

kernel:
	cargo build -vv $(build_args)

$(bin): kernel
	$(objcopy) $(kernel) --strip-all -O binary $@
ifeq ($(mode), debug)
	$(objcopy) $(kernel) --only-keep-debug $(kernel_debuginfo)

#build_args += --features link_kdbg
export KDBG = $(kernel_debuginfo)

else ifeq ($(mode), release)
build_args += --release
endif

asm:
	$(objdump) -d $(kernel) | less

build: $(bin)

rcore-fs-fuse:
ifeq ($(shell which rcore-fs-fuse),)
	@echo Installing rcore-fs-fuse
	@cargo install rcore-fs-fuse --git https://github.com/rcore-os/rcore-fs --rev 6df6cd24
endif

$(simplefs_img): rcore-fs-fuse $(out_dir)
	@rcore-fs-fuse --fs sfs $@ $(out_dir) zip

user_img: $(simplefs_img)

clean:
	cargo clean

#
qemu:
	qemu-system-riscv64 \
		-machine virt \
		-no-reboot \
		-no-shutdown \
		-nographic \
		-bios default \
		-kernel $(bin)
		#-device loader,file=$(bin),addr=0x80200000 #addr is the key, or try "-kernel"

run: build qemu

build-thead: build

run-thead: build
	@cp C906/fw_jump-0x40020000.bin fw.bin
	@dd if=$(bin) of=fw.bin bs=1 seek=131072
	echo $(PWD)/fw.bin
	xfel ddr d1
	xfel write 0x40000000 fw.bin
	xfel exec 0x40000000

