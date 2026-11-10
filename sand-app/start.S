.section .text
.global _start
.type _start, @function

_start:
    /*
     * SAND OS currently exposes one test-only ring-3 syscall:
     *   EAX = 1
     *   EBX = 0x53414E44 ("SAND")
     *
     * The kernel reports the test result through its debug console.
     * This is deliberately NOT a public application syscall ABI.
     */
    movl $1, %eax
    movl $0x53414E44, %ebx
    int $0x80

1:
    jmp 1b

.size _start, .-_start
