#![forbid(unsafe_code)]

//! Shared internal contracts.
//!
//! This module is the single Rust representation of `docs/contracts.md`.
//! Every component that wants to control the lights produces a
//! [`LightingCommand`] and hands it to the Controller; components never talk
//! to the Regulation Service directly.

/// Priority of a lighting command.
///
/// Ordering follows the numeric contract: `Standard < Override < SigKill`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Standard = 10,
    Override = 50,
    SigKill = 100,
}

/// Command emitted by policy components and evaluated by the Controller.
#[derive(Debug, Clone)]
pub struct LightingCommand {
    pub priority: Priority,
    pub state: bool,
    pub dimming: u8,
    /// Colour temperature in Kelvin.
    pub color_temp: Option<u16>,
    /// RGB colour channels, ordered as (red, green, blue).
    pub rgb: Option<(u16, u16, u16)>,
    pub source: String,
}
