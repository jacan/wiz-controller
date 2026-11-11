#![forbid(unsafe_code)]

//! WiZ Controller services.
//!
//! The binary in `src/main.rs` is intentionally thin; the functional
//! components live here so they can be composed and tested independently.

pub mod contracts;
pub mod udp;
