.global _start

.section .multiboot
.align 4
.long 0x1BADB002         # Magic Multiboot
.long 0x00000002         # Flags: bit 1 = provide memory info
.long -(0x1BADB002 + 0x00000002)  # Checksum (magic + flags + checksum = 0)

.section .text
.align 4
_start:
    # GRUB passes: EAX = multiboot magic (0x2BADB002)
    #              EBX = pointer to multiboot info structure

    # Clear BSS (clobbers EAX, ECX, EDI — EBX is preserved)
    mov $__bss_start, %edi
    mov $__bss_end, %ecx
    sub %edi, %ecx
    shr $2, %ecx            # byte count → dword count
    xor %eax, %eax
    cld
    rep stosl

    # Setup stack (lives in BSS, now zeroed)
    mov $stack_top, %esp

    # Pass multiboot info pointer as argument to rust_main
    push %ebx
    call rust_main

halt:
    cli
    hlt
    jmp halt

.section .bss
.align 16
stack_bottom:
    .skip 16384            # 16 KB stack
stack_top:
