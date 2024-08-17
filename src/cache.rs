//! L1-Cache control
//!
//! # WARNING
//!
//! Tests needed.

#![allow(non_camel_case_types)]
#![allow(unused)]
use core::arch::asm;

/// cache line size is 64B
pub const L1C_CACHELINE_SIZE: usize = 64;

const MCACHE_CTL_IC_EN_MASK: usize = 1 << 0;
const MCACHE_CTL_DC_EN_MASK: usize = 1 << 1;
const MCACHE_CTL_IPREF_EN_MASK: usize = 1 << 9;
const MCACHE_CTL_DPREF_EN_MASK: usize = 1 << 10;
const MCACHE_CTL_DC_WAROUND_MASK: usize = 3 << 13;

const MMSC_CCTL_VERSION_MASK: usize = 3 << 18;

#[repr(usize)]
#[derive(Clone, Copy)]
enum CacheControlCommand {
    L1D_VA_INVAL = 0,
    L1D_VA_WB = 1,
    L1D_VA_WBINVAL = 2,
    L1D_VA_LOCK = 3,
    L1D_VA_UNLOCK = 4,
    L1D_WBINVAL_ALL = 6,
    L1D_WB_ALL = 7,
    L1I_VA_INVAL = 8,
    L1I_VA_LOCK = 11,
    L1I_VA_UNLOCK = 12,
    L1D_IX_INVAL = 16,
    L1D_IX_WB = 17,
    L1D_IX_WBINVAL = 18,
    L1D_IX_RTAG = 19,
    L1D_IX_RDATA = 20,
    L1D_IX_WTAG = 21,
    L1D_IX_WDATA = 22,
    L1D_INVAL_ALL = 23,
    L1I_IX_INVAL = 24,
    L1I_IX_RTAG = 27,
    L1I_IX_RDATA = 28,
    L1I_IX_WTAG = 29,
    L1I_IX_WDATA = 30,
}

macro_rules! read_csr {
    ($csr_number:literal) => {{
        let r: usize;
        asm!(concat!("csrr {0}, ", stringify!($csr_number)), out(reg) r);
        r
    }}
}

macro_rules! write_csr {
    ($csr_number:literal, $bits:ident) => {{
        let bits: usize = $bits;
        asm!(concat!("csrw ", stringify!($csr_number), ", {0}"), in(reg) bits)
    }}
}

macro_rules! assert_address_size {
    ($ADDR:ident, $SIZE:ident) => {
        assert!(($ADDR % L1C_CACHELINE_SIZE == 0) && ($SIZE % L1C_CACHELINE_SIZE == 0));
    };
}

#[inline(always)]
unsafe fn set_mcache(bit: usize) {
    asm!("csrs 0x7ca, {0}", in(reg) bit);
}

#[inline(always)]
unsafe fn clear_mcache(bit: usize) {
    asm!("csrc 0x7ca, {0}", in(reg) bit);
}

#[inline(always)]
unsafe fn read_mcache() -> usize {
    read_csr!(0x7ca)
}

#[inline(always)]
unsafe fn l1c_cctl_cmd(cmd: usize) {
    write_csr!(0x7cc, cmd);
}

#[inline(always)]
unsafe fn l1c_cctl_address(addr: usize) {
    write_csr!(0x7cb, addr);
}

#[inline(always)]
unsafe fn l1c_cctl_read_address() -> usize {
    read_csr!(0x7cb)
}

unsafe fn l1c_op(opcode: CacheControlCommand, address: usize, size: usize) {
    let mut next_address;

    if (read_csr!(0xfc2) & MMSC_CCTL_VERSION_MASK != 0) {
        l1c_cctl_address(address);
        next_address = address;
        while (next_address < (address + size)) && (next_address >= address) {
            l1c_cctl_cmd(opcode as usize);
            next_address = l1c_cctl_read_address();
        }
    } else {
        for i in (0..size).step_by(L1C_CACHELINE_SIZE) {
            l1c_cctl_address(address + i);
            l1c_cctl_cmd(opcode as usize);
        }
    }
}

/// Check if I-Cache is enabled
#[inline(always)]
pub fn icache_is_enabled() -> bool {
    unsafe { read_mcache() & MCACHE_CTL_IC_EN_MASK == 1 }
}

/// Check if D-Cache is enabled
#[inline(always)]
pub fn dcache_is_enabled() -> bool {
    unsafe { read_mcache() & MCACHE_CTL_DC_EN_MASK == 1 }
}

/// Enable I-Cache
pub fn icache_enable() {
    if !icache_is_enabled() {
        unsafe {
            set_mcache(MCACHE_CTL_IC_EN_MASK | MCACHE_CTL_IPREF_EN_MASK);
        }
    }
}

/// Disable I-Cache
pub fn icache_disable() {
    if icache_is_enabled() {
        unsafe {
            clear_mcache(MCACHE_CTL_IC_EN_MASK);
        }
    }
}

/// Enable D-Cache
pub fn dcache_enable() {
    if !dcache_is_enabled() {
        unsafe {
            clear_mcache(MCACHE_CTL_DC_WAROUND_MASK);
            set_mcache(MCACHE_CTL_DC_EN_MASK);
        }
    }
}

/// Disable D-Cache
pub fn dcache_disable() {
    if dcache_is_enabled() {
        unsafe {
            clear_mcache(MCACHE_CTL_DC_EN_MASK);
        }
    }
}

/// Invalidate all D-Cache
pub fn dcache_invalidate_all() {
    unsafe {
        l1c_cctl_cmd(CacheControlCommand::L1D_INVAL_ALL as usize);
    }
}

/// Writeback all D-Cache
pub fn dcache_writeback_all() {
    unsafe {
        l1c_cctl_cmd(CacheControlCommand::L1D_WB_ALL as usize);
    }
}

/// Flush all D-Cache
pub fn dcache_flush_all() {
    unsafe {
        l1c_cctl_cmd(CacheControlCommand::L1D_WBINVAL_ALL as usize);
    }
}

/// D-Cache fill and lock by address
pub fn dcache_fill_lock(address: usize, size: usize) {
    assert_address_size!(address, size);
    unsafe {
        l1c_op(CacheControlCommand::L1D_VA_LOCK, address, size);
    }
}

/// D-Cache invalidate by address
pub fn dcache_invalidate(address: usize, size: usize) {
    assert_address_size!(address, size);
    unsafe {
        l1c_op(CacheControlCommand::L1D_VA_INVAL, address, size);
    }
}

/// D-Cache writeback by address
pub fn dcache_writeback(address: usize, size: usize) {
    assert_address_size!(address, size);
    unsafe {
        l1c_op(CacheControlCommand::L1D_VA_WB, address, size);
    }
}

/// I-Cache fill and lock by address
pub fn icache_fill_lock(address: usize, size: usize) {
    assert_address_size!(address, size);
    unsafe {
        l1c_op(CacheControlCommand::L1I_VA_LOCK, address, size);
    }
}

/// I-Cache invalidate by address
pub fn icache_invalidate(address: usize, size: usize) {
    assert_address_size!(address, size);
    unsafe {
        l1c_op(CacheControlCommand::L1I_VA_INVAL, address, size);
    }
}
