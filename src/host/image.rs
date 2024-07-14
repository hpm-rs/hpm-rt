#![allow(unused)]

use std::error::Error;
use std::fmt::Display;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

use super::device::{Device, Memory, MemoryType};
use super::linker;

const DEFAULT_STACK_SIZE: usize = 8 * 1024;

/// Flash type
#[derive(Clone, Copy)]
pub enum FlashType {
    /// SFDP SDR
    SfdpSdr,
    /// SFDP DDR
    SfdpDdr,
    /// 1-4-4 Read by 0xEB
    Read144,
    /// 1-2-2 Read by 0xBB
    Read122,
    /// HyperBus 1V8
    HyperBus1v8,
    /// HyperBus 3V3
    HyperBus3v3,
    /// OctaBus DDR
    OctaBusDdr,
    /// Xccela DDR
    XccelaDdr,
    /// EcoXiP DDR
    EcoXipDdr,
}

/// Flash interface type
#[derive(Clone, Copy)]
pub enum FlashInterface {
    /// Standard SPI
    Standard,
    /// Dual SPI
    Dual,
    /// Quad SPI
    Quad,
    /// Octa SPI
    Octa,
}

/// Quad I/O enable sequence
#[derive(Clone, Copy)]
pub enum QuadIOEnableSequence {
    /// Don't need or auto
    None,
    /// QE bit is at bit 6 in Status Register 1
    Status1Bit6,
    /// QE bit is at bit 1 in Status Register 2
    Status2Bit1,
    /// QE bit is at bit 7 in Status Register 2
    Status2Bit7,
    /// QE bit is at bit 1 in Status Register 2 and should be programmed by 0x31
    Status2Bit1ProgrammedBy0x31,
}

/// Flash I/O Voltage
#[derive(Clone, Copy)]
pub enum IOVoltage {
    /// IO voltage 3.3V
    Voltage3v3,
    /// IO voltage 1.8V
    Voltage1v8,
}

/// XPI pin group
#[derive(Clone, Copy)]
pub enum PinGroup {
    /// Group 1
    Group1,
    /// Group 2
    Group2,
}

/// XPI connection type
#[derive(Clone, Copy)]
pub enum PortConnection {
    /// Port A with CS0
    PortACs0,
    /// Port B with CS0
    PortBCs0,
    /// Port A with CS0 + Port B with CS0
    PortACs0PortBCs0,
    /// Port A with CS0 + Port A with CS1
    PortACs0PortACs1,
    /// Port B with CS1 + Port B with CS1
    PortBCs0PortBCs1,
}

/// Sector erase size
#[derive(Clone, Copy)]
pub enum SectorEraseSize {
    /// 4 KByes
    Erase4KB,
    /// 32 KByes
    Erase32KB,
    /// 64 KByes
    Erase64KB,
    /// 256 KByes
    Erase256KB,
}

/// Sector size
#[derive(Clone, Copy)]
pub enum SectorSize {
    /// 4 KByes
    Size4KB,
    /// 32 KByes
    Size32KB,
    /// 64 KByes
    Size64KB,
    /// 256 KByes
    Size256KB,
}

/// Flash size
#[derive(Clone, Copy)]
pub enum FlashSize {
    /// 4 MBytes
    Size4MB,
    /// 8 MBytes
    Size8MB,
    /// 16 MBytes
    Size16MB,
}

/// Indicate which XPI instance is used
#[derive(Clone, Copy, PartialEq)]
pub enum Instance {
    /// XPI 0
    Xpi0,
    /// XPI 1
    Xpi1,
}

impl Instance {
    /// Convert `Instance` to [`&str`]
    pub fn as_str(&self) -> &str {
        match self {
            Instance::Xpi0 => "XPI0",
            Instance::Xpi1 => "XPI1",
        }
    }
}

impl From<Instance> for MemoryType {
    fn from(value: Instance) -> Self {
        match value {
            Instance::Xpi0 => MemoryType::Xpi0,
            Instance::Xpi1 => MemoryType::Xpi0,
        }
    }
}

/// XPI NOR flash configuration info
#[derive(Clone, Copy)]
pub struct XpiNorConfigurationOption {
    flash_type: FlashType,
    quad_io_enable_sequence: QuadIOEnableSequence,
    pin_group: PinGroup,
    connect_port: PortConnection,
    instance: Instance,
}

impl XpiNorConfigurationOption {
    const DEFAULT_CONFIGURATION: [u32; 3] = [0xFCF90002, 0x00000007, 0x0];

    /// Create a default XPI NOR configuration info
    pub fn new() -> Self {
        Self {
            flash_type: FlashType::SfdpSdr,
            quad_io_enable_sequence: QuadIOEnableSequence::None,
            pin_group: PinGroup::Group1,
            connect_port: PortConnection::PortACs0,
            instance: Instance::Xpi0,
        }
    }

    /// Set flash type of connected to XPI
    pub fn flash_type(mut self, flash_type: FlashType) -> Self {
        self.flash_type = flash_type;
        self
    }

    /// Set Quad I/O enable sequence
    pub fn quad_io_enable_sequence(mut self, sequence: QuadIOEnableSequence) -> Self {
        self.quad_io_enable_sequence = sequence;
        self
    }

    /// Set XPI pin group used to connect to flash
    pub fn pin_group(mut self, group: PinGroup) -> Self {
        self.pin_group = group;
        self
    }

    /// Set XPI port used to connect to flash
    pub fn connect_port(mut self, port: PortConnection) -> Self {
        self.connect_port = port;
        self
    }

    /// Write configuration as bytes into vector etc.
    ///
    /// # Errors
    ///
    /// This function will return the error that [`Write::write_all`] returns.
    pub fn write(&self, writer: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        let mut conf = Self::DEFAULT_CONFIGURATION;

        conf[1] |= (self.flash_type as u32) << 28;
        conf[1] |= (self.quad_io_enable_sequence as u32) << 16;
        conf[2] |= (self.pin_group as u32) << 12;
        conf[2] |= (self.connect_port as u32) << 8;

        unsafe {
            writer.write_all(&core::mem::transmute::<[u32; 3], [u8; 12]>(conf))?;
        }
        Ok(())
    }
}

pub(crate) struct Region {
    pub(crate) memory: MemoryType,
    pub(crate) load_memory: Option<MemoryType>,
}

/// Boot Image builder
pub struct RuntimeBuilder {
    device: Device,
    xpi_nor_conf_info: Option<XpiNorConfigurationOption>,
    text: Region,
    rodata: Region,
    data: Region,
    bss: Region,
    stack: Region,
    heap: Region,
    stack_size: usize,
}

impl RuntimeBuilder {
    const DEFAULT_LINKER_SCRIPT_NAME: &'static str = "hpmrt-link.ld";

    /// Create [`RuntimeBuilder`] that boot from XPI.
    pub fn from_flash(family: Device, xpi_config: XpiNorConfigurationOption) -> Self {
        let boot_flash: MemoryType = xpi_config.instance.into();
        let mut builder = Self {
            device: family,
            xpi_nor_conf_info: Some(xpi_config),
            text: Region {
                memory: MemoryType::Xpi0,
                load_memory: Some(boot_flash),
            },
            rodata: Region {
                memory: MemoryType::Xpi0,
                load_memory: Some(boot_flash),
            },
            data: Region {
                memory: MemoryType::Dlm,
                load_memory: Some(boot_flash),
            },
            bss: Region {
                memory: MemoryType::Dlm,
                load_memory: None,
            },
            stack: Region {
                memory: MemoryType::Dlm,
                load_memory: None,
            },
            heap: Region {
                memory: MemoryType::Dlm,
                load_memory: None,
            },
            stack_size: DEFAULT_STACK_SIZE,
        };

        builder
    }

    /// Create [`RuntimeBuilder`] that boot from ILM.
    pub fn from_ram(device: Device) -> Self {
        Self {
            device,
            xpi_nor_conf_info: None,
            text: Region {
                memory: MemoryType::Ilm,
                load_memory: Some(MemoryType::Ilm),
            },
            rodata: Region {
                memory: MemoryType::Ilm,
                load_memory: Some(MemoryType::Ilm),
            },
            data: Region {
                memory: MemoryType::Dlm,
                load_memory: Some(MemoryType::Ilm),
            },
            bss: Region {
                memory: MemoryType::Dlm,
                load_memory: None,
            },
            stack: Region {
                memory: MemoryType::Dlm,
                load_memory: None,
            },
            heap: Region {
                memory: MemoryType::Dlm,
                load_memory: None,
            },
            stack_size: DEFAULT_STACK_SIZE,
        }
    }

    /// Set the size of the flash connected to XPI0
    pub fn xpi0_flash_size(mut self, size: u32) -> Self {
        let xpi0 = self
            .device
            .xpi0
            .as_mut()
            .unwrap_or_else(|| panic!("device does not have XPI0"));

        xpi0.size = size;
        self
    }

    /// Set the size of the flash connected to XPI1
    pub fn xpi1_flash_size(mut self, size: u32) -> Self {
        let xpi1 = self
            .device
            .xpi1
            .as_mut()
            .unwrap_or_else(|| panic!("device does not have XPI1"));

        xpi1.size = size;
        self
    }

    /// Specify where to place the `.rodata` section
    pub fn rodata(mut self, memory: MemoryType) -> Self {
        self.rodata.memory = memory;
        self
    }

    /// Specify where to place the `.data` section
    pub fn data(mut self, memory: MemoryType) -> Self {
        self.data.memory = memory;
        self
    }

    /// Specify where to place the `.bss` section
    pub fn bss(mut self, memory: MemoryType) -> Self {
        self.bss.memory = memory;
        self
    }

    /// Specify where to place the stack region
    pub fn stack(mut self, memory: MemoryType, size: usize) -> Self {
        self.stack.memory = memory;
        self.stack_size = size;
        self
    }

    /// Commit the runtime configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if run out of a build script or [`fs::write`] returns.
    pub fn build(self) -> Result<(), Box<dyn std::error::Error>> {
        // Since `build` is called from a build script, the output directory
        // represents the path to the _user's_ crate.
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        println!("cargo:rustc-link-search={}", out_dir.display());
        println!(
            "cargo:rustc-link-arg=-T{}",
            Self::DEFAULT_LINKER_SCRIPT_NAME
        );

        let mut in_memory = Vec::new();
        self.write_linker_script(&mut in_memory)?;
        fs::write(out_dir.join(Self::DEFAULT_LINKER_SCRIPT_NAME), &in_memory)?;
        Ok(())
    }

    fn check_section_placement(&self, region: &Region, name: &str) -> Result<(), String> {
        let memories = [
            &region.memory,
            &region.load_memory.unwrap_or(MemoryType::Ilm),
        ];
        for memory in memories {
            if !match memory {
                MemoryType::Ilm => self.device.ilm.is_some(),
                MemoryType::Dlm => self.device.dlm.is_some(),
                MemoryType::AxiSram0 => self.device.axi_sram_0.is_some(),
                MemoryType::AxiSram1 => self.device.axi_sram_0.is_some(),
                MemoryType::AhbSram => self.device.ahb_sram.is_some(),
                MemoryType::ApbSram => self.device.apb_sram.is_some(),
                MemoryType::Xpi0 => self.device.xpi0.is_some(),
                MemoryType::Xpi1 => self.device.xpi1.is_some(),
                _ => true,
            } {
                return Err(format!(
                    "{} not specified but used by region {}",
                    region.memory, name
                ));
            }
        }
        Ok(())
    }

    fn write_linker_script(
        &self,
        writer: &mut dyn Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut memories = Vec::new();

        macro_rules! check_memory {
            ($MEMORY:expr) => {
                if let Some(mem) = $MEMORY {
                    memories.push(mem);
                }
            };
        }

        // Collect memories
        check_memory!(&self.device.ilm);
        check_memory!(&self.device.dlm);
        check_memory!(&self.device.axi_sram_0);
        check_memory!(&self.device.axi_sram_1);
        check_memory!(&self.device.ahb_sram);
        check_memory!(&self.device.apb_sram);
        check_memory!(&self.device.xpi0);
        check_memory!(&self.device.xpi1);

        // Memory
        linker::write_memory(&memories, writer)?;

        // Region alias
        self.check_section_placement(&self.text, "TEXT")?;
        self.check_section_placement(&self.rodata, "RODATA")?;
        self.check_section_placement(&self.data, "DATA")?;
        self.check_section_placement(&self.stack, "STACK")?;
        self.check_section_placement(&self.heap, "HEAP")?;

        linker::region_alias(self.text.memory, "TEXT", writer)?;
        linker::region_alias(self.rodata.memory, "RODATA", writer)?;
        linker::region_alias(self.data.memory, "DATA", writer)?;
        linker::region_alias(self.bss.memory, "BSS", writer)?;
        linker::region_alias(self.stack.memory, "STACK", writer)?;
        linker::region_alias(self.heap.memory, "HEAP", writer)?;
        linker::region_alias(self.text.load_memory.unwrap(), "LOAD_TEXT", writer)?;
        linker::region_alias(self.rodata.load_memory.unwrap(), "LOAD_RODATA", writer)?;
        linker::region_alias(self.data.load_memory.unwrap(), "LOAD_DATA", writer)?;

        writeln!(writer, "PROVIDE(_stack_size = {});", self.stack_size)?;

        if let Some(xpi_nor_conf_info) = self.xpi_nor_conf_info {
            let mut bytes: [u32; 3] = [0; 3];

            // XPI NOR configure option
            unsafe {
                xpi_nor_conf_info.write(&mut core::slice::from_raw_parts_mut(
                    bytes.as_mut_ptr() as *mut u8,
                    core::mem::size_of_val(&bytes),
                ));
            }

            linker::region_alias(xpi_nor_conf_info.instance.into(), "BOOT_FLASH", writer)?;
            linker::output_bytes(
                ".nor_cfg_option",
                "ORIGIN(REGION_BOOT_FLASH)",
                0x400,
                &bytes,
                "REGION_BOOT_FLASH",
                writer,
            )?;

            // Boot header
            linker::output_boot_header(writer)?;
            writeln!(writer, "PROVIDE(_stext = __app_load_addr__);")?;
        } else {
            writeln!(writer, "PROVIDE(_stext = ORIGIN(REGION_TEXT));")?;
        }

        let link_x = include_bytes!("linker/hpmrt-link.x");
        writer.write_all(link_x)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Family, RuntimeBuilder, XpiNorConfigurationOption};

    #[test]
    pub fn write_memory() {
        let mut stdout = std::io::stdout();

        RuntimeBuilder::from_flash(Family::HPM6700_6400, XpiNorConfigurationOption::new())
            .xpi0_flash_size(512 * 1024)
            .write_linker_script(&mut stdout)
            .unwrap();
    }
}
