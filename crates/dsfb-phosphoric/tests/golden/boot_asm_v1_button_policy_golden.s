.text

.p2align 4
phosphoric_debug_puts:
    pushq %rbp
    movq %rsp, %rbp
    movq %rcx, %r8
    movw $1026, %dx
.Lphosphoric_debug_puts_next:
    movb (%r8), %al
    testb %al, %al
    je .Lphosphoric_debug_puts_done
    outb %al, %dx
    incq %r8
    jmp .Lphosphoric_debug_puts_next
.Lphosphoric_debug_puts_done:
    leave
    retq

# phosphoric_efi_puts(systab: rcx, ucs2_msg: rdx)
# Calls ConOut->OutputString so text appears on the firmware framebuffer
# (visible in QEMU screendumps). SystemTable layout per UEFI 2.x spec:
#   SystemTable + 0x40  =  ConOut (EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *)
#   ConOut      + 0x08  =  OutputString (fn ptr)
# OutputString(this, string) uses Microsoft x64 calling convention:
#   rcx = this (ConOut*),  rdx = string (CHAR16*)
.p2align 4
phosphoric_efi_puts:
    pushq %rbp
    movq %rsp, %rbp
    subq $32, %rsp                 # 32-byte shadow space for callee
    testq %rcx, %rcx
    je .Lphosphoric_efi_puts_done
    movq 0x40(%rcx), %rax           # rax = ConOut
    testq %rax, %rax
    je .Lphosphoric_efi_puts_done
    movq %rax, %rcx                 # rcx = ConOut (this)
    # rdx already holds the UCS-2 string
    callq *0x08(%rax)               # ConOut->OutputString(this, string)
.Lphosphoric_efi_puts_done:
    addq $32, %rsp
    leave
    retq

.p2align 4
phosphoric_demo_v1__button_next_pressed:
    pushq %rbp
    movq %rsp, %rbp
    movzwl %dx, %eax
    cmpl $32, %eax
    jne .Lbutton_next_keep
    movq %rcx, %rax
    xorq $1, %rax
    andq $1, %rax
    leave
    retq
.Lbutton_next_keep:
    movq %rcx, %rax
    andq $1, %rax
    leave
    retq

.p2align 4
phosphoric_demo_v1__button_redraw_requested:
    pushq %rbp
    movq %rsp, %rbp
    movzwl %cx, %eax
    cmpl $32, %eax
    sete %al
    movzbl %al, %eax
    andq $1, %rax
    leave
    retq

.def phosphoric_demo_init;
    .scl 2;
    .type 32;
.endef
.globl phosphoric_demo_init
.p2align 4
phosphoric_demo_init:
    pushq %rbp
    movq %rsp, %rbp
    movl (%rdx), %r8d
    movl %r8d, 0(%rcx)
    movl 4(%rdx), %r8d
    movl %r8d, 4(%rcx)
    movl 8(%rdx), %r8d
    movl %r8d, 8(%rcx)
    movl 12(%rdx), %r8d
    movl %r8d, 12(%rcx)
    movl 16(%rdx), %r8d
    movl %r8d, 16(%rcx)
    movl $48, 20(%rcx)
    movl $48, 24(%rcx)
    movl $220, 28(%rcx)
    movl $140, 32(%rcx)
    movl $72, 36(%rcx)
    movl $72, 40(%rcx)
    movl $0, 44(%rcx)
    movl $0, 48(%rcx)
    movl $1, 52(%rcx)
    movl $0, 56(%rcx)
    movq %rcx, %rax
    leave
    retq

.def phosphoric_demo_step;
    .scl 2;
    .type 32;
.endef
.globl phosphoric_demo_step
.p2align 4
phosphoric_demo_step:
    pushq %rbp
    movq %rsp, %rbp
    subq $48, %rsp
    movq %rcx, -8(%rbp)
    movq %r8, -16(%rbp)
    movq 0(%rdx), %r9
    movq %r9, 0(%rcx)
    movq 8(%rdx), %r9
    movq %r9, 8(%rcx)
    movq 16(%rdx), %r9
    movq %r9, 16(%rcx)
    movq 24(%rdx), %r9
    movq %r9, 24(%rcx)
    movq 32(%rdx), %r9
    movq %r9, 32(%rcx)
    movq 40(%rdx), %r9
    movq %r9, 40(%rcx)
    movq 48(%rdx), %r9
    movq %r9, 48(%rcx)
    movl 56(%rdx), %r9d
    movl %r9d, 56(%rcx)
    movl $0, 60(%rcx)
    movl $0, 64(%rcx)
    movl $0, 68(%rcx)
    movl $0, 72(%rcx)
    movl (%r8), %r9d
    cmpl $0, %r9d
    jne .Lboot_demo_step_done
    movl $1, 48(%rcx)
    movl $1, 60(%rcx)
    movq -8(%rbp), %r10
    movq -16(%rbp), %r8
    movl 44(%r10), %ecx
    andl $1, %ecx
    movzwl 4(%r8), %edx
    movzwl 6(%r8), %r8d
    call phosphoric_demo_v1__button_next_pressed
    andl $1, %eax
    movl %eax, -20(%rbp)
    movq -16(%rbp), %r8
    movzwl 4(%r8), %ecx
    movzwl 6(%r8), %edx
    call phosphoric_demo_v1__button_redraw_requested
    andl $1, %eax
    movq -8(%rbp), %rcx
    movl -20(%rbp), %r9d
    movl %r9d, 44(%rcx)
    movl %eax, 64(%rcx)
    movl %eax, 68(%rcx)
.Lboot_demo_step_done:
    movq -8(%rbp), %rax
    leave
    retq

.def phosphoric_demo_render;
    .scl 2;
    .type 32;
.endef
.globl phosphoric_demo_render
.p2align 4
phosphoric_demo_render:
    pushq %rbp
    movq %rsp, %rbp
    movl $1, 0(%rcx)
    movl $0, 4(%rcx)
    movl $0, 8(%rcx)
    movl $0, 12(%rcx)
    movl 0(%rdx), %eax
    movl %eax, 16(%rcx)
    movl 4(%rdx), %eax
    movl %eax, 20(%rcx)
    movb $16, 24(%rcx)
    movb $36, 25(%rcx)
    movb $58, 26(%rcx)
    movb $0, 27(%rcx)
    movq %rcx, %rax
    leave
    retq

.def efi_main;
    .scl 2;
    .type 32;
.endef
.globl efi_main
.p2align 4
efi_main:
    pushq %rbp
    movq %rsp, %rbp
    subq $1056, %rsp
    # UEFI calling convention: rcx = ImageHandle, rdx = SystemTable*
    movq %rdx, -8(%rbp)             # save SystemTable*
    leaq msg_entering(%rip), %rcx
    call phosphoric_debug_puts
    movq -8(%rbp), %rcx
    leaq efi_msg_entering(%rip), %rdx
    call phosphoric_efi_puts
    leaq msg_runtime_active(%rip), %rcx
    call phosphoric_debug_puts
    movq -8(%rbp), %rcx
    leaq efi_msg_runtime_active(%rip), %rdx
    call phosphoric_efi_puts
    movl $500, -32(%rbp)
    movl $320, -28(%rbp)
    movl $500, -24(%rbp)
    movl $4, -20(%rbp)
    movl $1, -16(%rbp)
    leaq -128(%rbp), %rcx
    leaq -32(%rbp), %rdx
    call phosphoric_demo_init
    movl $0, -192(%rbp)
    movw $32, -188(%rbp)
    movw $0, -186(%rbp)
    movl $0, -184(%rbp)
    movl $0, -180(%rbp)
    movl $0, -176(%rbp)
    leaq -320(%rbp), %rcx
    leaq -128(%rbp), %rdx
    leaq -192(%rbp), %r8
    call phosphoric_demo_step
    cmpl $0, -260(%rbp)
    je .Lefi_boot_fail
    leaq msg_event_routed(%rip), %rcx
    call phosphoric_debug_puts
    movq -8(%rbp), %rcx
    leaq efi_msg_event_routed(%rip), %rdx
    call phosphoric_efi_puts
    leaq -768(%rbp), %rcx
    leaq -320(%rbp), %rdx
    call phosphoric_demo_render
    cmpl $0, -768(%rbp)
    je .Lefi_boot_fail
    leaq msg_redraw_complete(%rip), %rcx
    call phosphoric_debug_puts
    movq -8(%rbp), %rcx
    leaq efi_msg_redraw_complete(%rip), %rdx
    call phosphoric_efi_puts
    leaq msg_demo_complete(%rip), %rcx
    call phosphoric_debug_puts
    movq -8(%rbp), %rcx
    leaq efi_msg_demo_complete(%rip), %rdx
    call phosphoric_efi_puts
    # Hold the firmware screen visible for ~5 seconds before exiting.
    # This gives the framebuffer screendump path a chance to capture
    # the post-render firmware-text frame.
    movq $1500000000, %rcx
.Lefi_visual_hold:
    decq %rcx
    jne .Lefi_visual_hold
    movw $244, %dx
    xorl %eax, %eax
    outl %eax, %dx
.Lefi_boot_halt:
    hlt
    jmp .Lefi_boot_halt
.Lefi_boot_fail:
    leaq msg_failed(%rip), %rcx
    call phosphoric_debug_puts
    movw $244, %dx
    movl $1, %eax
    outl %eax, %dx
    jmp .Lefi_boot_halt

.section .rdata,"dr"
msg_entering:
    .asciz "phosphoric: entering generated boot-asm demo\r\n"
msg_runtime_active:
    .asciz "phosphoric: generated boot-asm demo runtime active\r\n"
msg_event_routed:
    .asciz "phosphoric: event routed\r\n"
msg_redraw_complete:
    .asciz "phosphoric: redraw complete\r\n"
msg_demo_complete:
    .asciz "phosphoric: demo complete\r\n"
msg_failed:
    .asciz "phosphoric: generated boot-asm demo failed\r\n"

# UCS-2 (UTF-16LE) strings for UEFI ConOut->OutputString. CHAR16 terminator
# is two zero bytes. Each printable ASCII char becomes two bytes (low,0).
.p2align 1
efi_msg_entering:
    .word 0x000A
    .word 'P, 'H, 'O, 'S, 'P, 'H, 'O, 'R, 'I, 'C, ' ', 'D, 'E, 'M, 'O, ' ', '*, ' '
    .word 'b, 'o, 'o, 't, ' ', 'r, 'u, 'n, 't, 'i, 'm, 'e, ' ', 'e, 'n, 't, 'e, 'r, 'e, 'd
    .word 0x000D
    .word 0x000A
    .word 0x0000
.p2align 1
efi_msg_runtime_active:
    .word 'P, 'H, 'O, 'S, 'P, 'H, 'O, 'R, 'I, 'C, ' ', 'D, 'E, 'M, 'O, ' ', '*, ' '
    .word 'g, 'e, 'n, 'e, 'r, 'a, 't, 'e, 'd, ' ', 'b, 'o, 'o, 't, '-, 'a, 's, 'm
    .word ' ', 'r, 'u, 'n, 't, 'i, 'm, 'e, ' ', 'a, 'c, 't, 'i, 'v, 'e
    .word 0x000D
    .word 0x000A
    .word 0x0000
.p2align 1
efi_msg_event_routed:
    .word 'P, 'H, 'O, 'S, 'P, 'H, 'O, 'R, 'I, 'C, ' ', 'D, 'E, 'M, 'O, ' ', '*, ' '
    .word 'i, 'n, 'p, 'u, 't, ' ', 'e, 'v, 'e, 'n, 't, ' ', 'r, 'o, 'u, 't, 'e, 'd
    .word 0x000D
    .word 0x000A
    .word 0x0000
.p2align 1
efi_msg_redraw_complete:
    .word 'P, 'H, 'O, 'S, 'P, 'H, 'O, 'R, 'I, 'C, ' ', 'D, 'E, 'M, 'O, ' ', '*, ' '
    .word 'b, 'u, 't, 't, 'o, 'n, '-, 'p, 'r, 'e, 's, 's, ' ', 'r, 'e, 'd, 'r, 'a, 'w
    .word ' ', 'c, 'o, 'm, 'p, 'l, 'e, 't, 'e
    .word 0x000D
    .word 0x000A
    .word 0x0000
.p2align 1
efi_msg_demo_complete:
    .word 'P, 'H, 'O, 'S, 'P, 'H, 'O, 'R, 'I, 'C, ' ', 'D, 'E, 'M, 'O, ' ', '*, ' '
    .word 'd, 'e, 'm, 'o, ' ', 'c, 'o, 'm, 'p, 'l, 'e, 't, 'e
    .word 0x000D
    .word 0x000A
    .word 0x0000
