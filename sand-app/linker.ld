OUTPUT_FORMAT(elf32-i386)
OUTPUT_ARCH(i386)
ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5); /* Read + execute; never writable + executable. */
}

SECTIONS
{
    . = 0x40000000;

    .text : ALIGN(4096)
    {
        *(.text*)
    } :text

    /DISCARD/ :
    {
        *(.comment)
        *(.note*)
        *(.eh_frame*)
        *(.data*)
        *(.bss*)
    }
}
