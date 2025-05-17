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

.PHONY: boot
boot:
	as --32 boot/boot.s -o boot/boot.o

.PHONY: link
link: kernel boot
	ld -m elf_i386 -Ttext 0x7c00 --oformat binary -o kernel.bin boot/boot.o target/i686-custom/release/libtacos.a

kernel.bin: link

.PHONY: run
run: kernel.bin
	qemu-system-i386 -fda kernel.bin -display curses

.PHONY: clean
clean:
	rm -f boot/boot.o kernel.bin
	cargo clean
