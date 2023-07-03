//! Minimal startup / runtime for HPMicro MCUs
//!
//! # Getting Start
//!
//! ```text
//! cargo add hpm-rt --build
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
