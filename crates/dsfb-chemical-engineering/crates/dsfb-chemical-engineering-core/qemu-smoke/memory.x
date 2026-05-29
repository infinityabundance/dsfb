/* Memory layout for the QEMU `lm3s6965evb` machine (Cortex-M3): 256K flash @ 0x0, 64K SRAM @ 0x20000000. */
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  RAM   : ORIGIN = 0x20000000, LENGTH = 64K
}
