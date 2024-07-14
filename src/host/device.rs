use std::fmt::Display;

/// Memory definition
#[derive(Clone, Copy)]
pub struct Memory {
    pub(crate) mem_type: MemoryType,
    pub(crate) base: u32,
    pub(crate) size: u32,
}

macro_rules! size {
    ($SIZE:literal KBytes) => {
        $SIZE * 1024
    };
    ($SIZE:literal Bytes) => {
        $SIZE
    };
    ($SIZE:literal) => {
        $SIZE
    };
    ($SIZE:expr) => {
        $SIZE
    };
}

macro_rules! memory {
    ($TYPE:expr, $BASE:expr, $($tail:tt)*) => {
        Some(crate::host::device::Memory {
            mem_type: $TYPE,
            base: $BASE,
            size: size!($($tail)*),
        })
    };
}

/// Memory partitions.
///
/// in the final program. Note that the `RuntimeBuilder` only does limited
/// checks on memory placements. Generally, it's OK to place data in ILM,
/// and instructions in DLM; however, this isn't recommended for optimal
/// performance.
#[derive(Debug, Clone, Copy)]
pub enum MemoryType {
    /// Place the section in instruction local memory (ILM).
    Ilm,
    /// Place the section in data local memory (DLM).
    Dlm,
    /// Place the section in AXI SRAM 0
    AxiSram0,
    /// Place the section in AXI SRAM 1
    AxiSram1,
    /// Place the section in AHB SRAM
    AhbSram,
    /// Place the section in APB SRAM
    ApbSram,
    /// Place the section in external flash and access via XPI0 bus.
    Xpi0,
    /// Place the section in external flash and access via XPI1 bus.
    Xpi1,
}

impl MemoryType {
    fn as_str(&self) -> &str {
        match self {
            MemoryType::Ilm => "ILM",
            MemoryType::Dlm => "DLM",
            MemoryType::AxiSram0 => "AXI_SRAM_0",
            MemoryType::AxiSram1 => "AXI_SRAM_1",
            MemoryType::AhbSram => "AHB_SRAM",
            MemoryType::ApbSram => "APB_SRAM",
            MemoryType::Xpi0 => "XPI0",
            MemoryType::Xpi1 => "XPI1",
        }
    }
}

impl Display for MemoryType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Device definition
#[derive(Clone, Copy)]
pub struct Device {
    pub(crate) ilm: Option<Memory>,
    pub(crate) dlm: Option<Memory>,
    pub(crate) axi_sram_0: Option<Memory>,
    pub(crate) axi_sram_1: Option<Memory>,
    pub(crate) ahb_sram: Option<Memory>,
    pub(crate) apb_sram: Option<Memory>,
    pub(crate) xpi0: Option<Memory>,
    pub(crate) xpi1: Option<Memory>,
}

/// HPMicro MCU family memory info
#[allow(non_snake_case)]
pub mod Family {
    use super::{Device, MemoryType};

    /// HPM6700/6400 series.
    pub const HPM6700_6400: Device = Device {
        ilm: memory!(MemoryType::Ilm, 0x0000_0000, 256 KBytes),
        dlm: memory!(MemoryType::Dlm, 0x0008_0000, 256 KBytes),
        axi_sram_0: memory!(MemoryType::AxiSram0, 0x0108_0000, 512 KBytes),
        axi_sram_1: memory!(MemoryType::AxiSram1, 0x0110_0000, 512 KBytes),
        ahb_sram: memory!(MemoryType::AhbSram, 0xF030_0000, 32 KBytes),
        apb_sram: memory!(MemoryType::ApbSram, 0xF40F_0000, 8 KBytes),
        xpi0: memory!(MemoryType::Xpi0, 0x8000_0000, 0 KBytes),
        xpi1: memory!(MemoryType::Xpi1, 0x9000_0000, 0 KBytes),
    };

    /// HPM6300 series.
    pub const HPM6300: Device = Device {
        ilm: memory!(MemoryType::Ilm, 0x0000_0000, 128 KBytes),
        dlm: memory!(MemoryType::Dlm, 0x0008_0000, 128 KBytes),
        axi_sram_0: memory!(MemoryType::AxiSram0, 0x0108_0000, 512 KBytes),
        axi_sram_1: None,
        ahb_sram: memory!(MemoryType::AhbSram, 0xF030_0000, 32 KBytes),
        apb_sram: None,
        xpi0: memory!(MemoryType::Xpi0, 0x8000_0000, 0 KBytes),
        xpi1: memory!(MemoryType::Xpi1, 0x9000_0000, 0 KBytes),
    };
}
