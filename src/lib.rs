#![no_std]
#![allow(dead_code)]  // temporary solution to avoid warnings for unused functions

// Hardware-dependent modules — only compiled for the bare-metal target (os = "none")
#[cfg(target_os = "none")]
pub mod drivers;
#[cfg(target_os = "none")]
pub mod gdt;
#[cfg(target_os = "none")]
pub mod io;
#[cfg(target_os = "none")]
pub mod shell;

// Pure-logic modules (always compiled, testable on host)
pub mod klib;
