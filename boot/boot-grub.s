.global _start

# Multiboot header
.section .multiboot
.align 4
.long 0x1BADB002              # magic number Multiboot
.long 0x0                     # flags (0 = rien demandé à GRUB)
.long -(0x1BADB002)            # checksum (magic + flags + checksum == 0)

.section .text
    .org 510
    .byte 0x55, 0xAA

.section .bss
.align 16
stack_bottom:
    .skip 16384                # 16 KB stack
stack_top:
