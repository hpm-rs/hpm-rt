use core::arch::asm;

const MCACHE_CTL_IC_EN_MASK: usize = 1 << 0;
const MCACHE_CTL_DC_EN_MASK: usize = 1 << 1;
const MCACHE_CTL_IPREF_EN_MASK: usize = 1 << 9;
const MCACHE_CTL_DPREF_EN_MASK: usize = 1 << 10;
const MCACHE_CTL_DC_WAROUND_MASK: usize = 3 << 13;

#[inline]
unsafe fn set_mcache(bit: usize) {
    asm!("csrs 0x7ca, {0}", in(reg) bit);
}

#[inline]
unsafe fn clear_mcache(bit: usize) {
    asm!("csrc 0x7ca, {0}", in(reg) bit);
}

#[inline]
unsafe fn read_mcache() -> usize {
    let r: usize;
    asm!("csrr {0}, 0x7ca", out(reg) r);
    r
}

/// Check if I-Cache is enabled
#[inline]
pub fn icache_is_enabled() -> bool {
    unsafe { read_mcache() & MCACHE_CTL_IC_EN_MASK == 1 }
}

/// Check if D-Cache is enabled
#[inline]
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
            set_mcache(MCACHE_CTL_DPREF_EN_MASK | MCACHE_CTL_DC_EN_MASK);
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
