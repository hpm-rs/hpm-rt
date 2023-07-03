use std::io::{Result, Write};

use crate::MemoryType;

use super::device::Memory;

pub(crate) fn write_memory(memories: &[&Memory], writer: &mut dyn Write) -> Result<()> {
    writeln!(writer, "MEMORY \n{{")?;
    for &m in memories {
        writeln!(
            writer,
            "{} : ORIGIN = 0x{:08X}, LENGTH = 0x{:08X}",
            m.mem_type, m.base, m.size
        )?;
    }
    writeln!(writer, "}}")?;
    Ok(())
}

pub(crate) fn region_alias(memory: MemoryType, name: &str, writer: &mut dyn Write) -> Result<()> {
    writeln!(writer, "REGION_ALIAS(\"REGION_{}\", {});", name, memory)?;
    Ok(())
}

pub(crate) fn output_boot_header(writer: &mut dyn Write) -> Result<()> {
    write!(writer, "{}", include_str!("boot_header.x"))
}

pub(crate) fn output_bytes(
    section: &str,
    address: &str,
    offset: u32,
    bytes: &[u32],
    region_name: &str,
    writer: &mut dyn Write,
) -> Result<()> {
    writeln!(writer, "SECTIONS\n{{\n{} {}:\n{{", section, address)?;
    writeln!(writer, ". += 0x{:X};", offset)?;
    for &b in bytes {
        writeln!(writer, "LONG(0x{:08X});", b)?;
    }
    writeln!(writer, "}} > {}\n}}", region_name)?;
    Ok(())
}
