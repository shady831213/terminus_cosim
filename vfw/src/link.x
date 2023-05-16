MEMORY
{
  ILM : ORIGIN = 0, LENGTH = 4K
  DLM : ORIGIN = 4096, LENGTH = 16K
  GLOBAL: ORIGIN = 0x80000000, LENGTH = 1M
}

OUTPUT_ARCH(riscv)
ENTRY(_start)

REGION_ALIAS("REGION_CPU_BSS", ILM);
REGION_ALIAS("REGION_CPU_DATA", ILM);
REGION_ALIAS("REGION_MAILBOX", DLM);
REGION_ALIAS("REGION_STACK", DLM);
REGION_ALIAS("REGION_TEXT", GLOBAL);
REGION_ALIAS("REGION_RODATA", GLOBAL);
REGION_ALIAS("REGION_DATA", GLOBAL);
REGION_ALIAS("REGION_HEAP", GLOBAL);
REGION_ALIAS("REGION_BSS", GLOBAL);
REGION_ALIAS("REGION_SYNCED_BSS", GLOBAL);
REGION_ALIAS("REGION_SYNCED_DATA", GLOBAL);
REGION_ALIAS("REGION_GOT", GLOBAL);

SECTIONS
{
    /DISCARD/ : {
        *(.eh_frame .eh_frame_hdr,.riscv.attributes, .debug_*, .comment);
    }
}

PROVIDE(_provide_base = ORIGIN(GLOBAL)/2);
PROVIDE(_stack_size = _provide_base + 3K);
PROVIDE(_heap_size = _provide_base + 4k);
PROVIDE(_num_cores = _provide_base + 3);
