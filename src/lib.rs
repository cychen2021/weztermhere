//! `weztermhere`: an "Open WezTerm here" entry for the Windows 11 context menu.
//!
//! The DLL built from this crate implements the `IExplorerCommand` COM class that the sparse package in `packaging/` registers with File Explorer.

pub mod launch;
pub mod shell;
