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
#   kernel                   # build the kernel
#   boot                     # assemble the bootloader
#   link                     # link the kernel and bootloader
#   iso                      # create an ISO image of the kernel
#   run                      # run the kernel in QEMU
#   clean                    # clean build artifacts
#   fclean                   # clean build artifacts and cargo cache
#   check-tools              # check if all required tools are installed
#   setup-42-linux           # setup the 42 Linux environment (compile and install everything fine...except grub-pc still missing)
#   vm-42-start              # start the Vagrant VM
#   vm-42-delete             # delete the Vagrant VM
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

.PHONY: fclean
fclean: clean
	cargo clean

.PHONY: check-tools
check-tools:
	@command -v cargo >/dev/null 2>&1 || { echo >&2 "cargo is not installed. Aborting."; exit 1; }
	@command -v as >/dev/null 2>&1 || { echo >&2 "GNU as is not installed. Aborting."; exit 1; }
	@command -v ld >/dev/null 2>&1 || { echo >&2 "ld is not installed. Aborting."; exit 1; }
	@command -v grub-mkrescue >/dev/null 2>&1 || { echo >&2 "grub-mkrescue is not installed. Aborting."; exit 1; }
	@command -v qemu-system-i386 >/dev/null 2>&1 || { echo >&2 "qemu-system-i386 is not installed. Aborting."; exit 1; }

.PHONY: setup-42-linux
setup-42-linux:
	@./setup-42-linux.sh

vm-42-start:
	@VBoxManage list systemproperties | grep "Default machine folder:"
	@vboxmanage setproperty machinefolder ~/sgoinfre
	@VBoxManage list systemproperties | grep "Default machine folder:"
	@vagrant up

vm-42-delete:
	@vagrant destroy -f
