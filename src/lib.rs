//! Minimal startup / runtime for HPMicro MCUs.
//!
//! # Getting Start
//!
//! ```text
//! cargo add hpm-rt --build
//! ```
//!
//! # Example
//!
//! Run in RAM, usually used when debugging.
//!
//! ```no_run
//! // build.rs
//! use hpm_rt::*;
//!
//! fn main() {
//!     RuntimeBuilder::from_ram(Family::HPM6700_6400)
//!         .build()
//!         .unwrap();
//!
//!     println!("cargo:rerun-if-changed=build.rs");
//! }
//! ```
//!
//! Here is a minimal example of booting from flash on the HPM6750EVKMINI board.
//!
//! ```no_run
//! // build.rs
//! use hpm_rt::*;
//!
//! fn main() {
//!     let xpi_nor_cfg = XpiNorConfigurationOption::new();
//!
//!     RuntimeBuilder::from_flash(Family::HPM6700_6400, xpi_nor_cfg)
//!         .xpi0_flash_size(8 * 1024 * 1024)
//!         .build()
//!         .unwrap();
//!
//!     println!("cargo:rerun-if-changed=build.rs");
//! }
//! ```

#![cfg_attr(all(target_arch = "riscv32", target_os = "none"), no_std)]
// NOTE: Adapted from cortex-m/src/lib.rs
#![deny(missing_docs)]

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "riscv32", target_os = "none"))] {
        mod target;
        /// L1-Cache control
        pub mod cache;

        pub use target::{entry, Interrupt};
    } else {
        mod host;

        pub use host::*;
    }
}
