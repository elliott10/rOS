target := riscv64imac-unknown-none-elf
mode := debug
kernel := target/$(target)/$(mode)/os
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
	cargo build

$(bin): kernel
	$(objcopy) $(kernel) --strip-all -O binary $@

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

