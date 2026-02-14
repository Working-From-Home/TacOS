.global _start

.section .multiboot
.align 4
.long 0x1BADB002         # Magic Multiboot
.long 0x0                # Flags (0 = nothing to do)
.long -(0x1BADB002)      # Checksum

.section .text
.align 4
_start:
    mov $stack_top, %esp   # Setup stack
    call rust_main         # Call rust function

halt:
    cli
    hlt
    jmp halt

.section .bss
.align 16
stack_bottom:
    .skip 16384            # 16 KB stack
stack_top:
