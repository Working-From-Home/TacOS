define ALL_HELP_INFO
########################################################
# A simple Makefile to help with common tasks.         #
########################################################
#
#
# Global targets:
# 
#   post-create              # run the post-create.sh of the devcontainer
#   build                    # build TacOS
endef

.PHONY: all
all: help

.PHONY: help
help:
	${ALL_HELP_INFO}

.PHONY: build
build:
	cargo rustc --release --target target-specs/i686-custom.json -- --emit=obj

.PHONY: kernel
kernel:
	cargo build --release

.PHONY: boot
boot:
	as --32 boot/boot.s -o boot/boot.o

.PHONY: link
link: build boot
	ld -m elf_i386 -T linker.ld -o kernel.elf boot/boot.o target/i686-custom/release/deps/*.o

.PHONY: iso
iso: link
	mkdir -p iso/boot/grub
	cp kernel.elf iso/boot/kernel.bin
	cp grub.cfg iso/boot/grub/grub.cfg
	grub-mkrescue -o tacos.iso iso

.PHONY: run
run: iso
	qemu-system-i386 -cdrom tacos.iso -display curses

.PHONY: clean
clean:
	rm -rf boot/boot.o iso/ kernel.elf tacos.iso
	cargo clean
