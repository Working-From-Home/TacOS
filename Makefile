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

.PHONY: setup
post-create:
	.devcontainer/scripts/post-create.sh

.PHONY: build
build: kernel.bin

.PHONY: kernel
kernel:
	cargo build --release

.PHONY: boot-grub
boot-grub:
	as --32 boot/boot-grub.s -o boot/boot-grub.o

.PHONY: link
link: kernel boot-grub
	ld -m elf_i386 -T linker.ld -o kernel.elf boot/boot-grub.o target/i686-custom/release/tacos

kernel.elf: link

.PHONY: iso
iso: kernel.elf
	mkdir -p iso/boot/grub
	cp kernel.elf iso/boot/kernel.bin
	cp grub.cfg iso/boot/grub/grub.cfg
	grub-mkrescue -o tacos.iso iso

.PHONY: run
run: iso
	qemu-system-i386 -cdrom tacos.iso -display curses

.PHONY: clean
clean:
	rm -fr boot/boot.o iso/ kernel.elf tacos.iso
	cargo clean
