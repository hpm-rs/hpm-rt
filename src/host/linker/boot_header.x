SECTIONS
{
  .boot_header :
  {
    . = ORIGIN(REGION_BOOT_FLASH) + 0x1000;
    __boot_header = .;

    KEEP(*(.boot_header));

    . = ORIGIN(REGION_BOOT_FLASH) + 0x3000;
    __app_load_addr__ = _stext;
    __app_offset__ = . - __boot_header;
  } > REGION_BOOT_FLASH
}
