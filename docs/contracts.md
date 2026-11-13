# Contracts & Data Structures

## 1. UDP Payload Protocol (WiZ Bulbs - Port 38899)
All communication happens via JSON over UDP to port 38899.

### Set Pilot Command (Example JSON)
```json
{
    "method": "setPilot",
    "params": {
        "state": true,
        "dimming": 30
    }
}
```

Colour is controlled by exactly one of:

- `temp`: white colour temperature in Kelvin
- `r`, `g`, `b`: direct RGB channels

A single `setPilot` command must not contain both `temp` and RGB values.

## 2. Internal Rust Contract
Every component that wants to control the lights must emit this exact struct to the Controller queue:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Standard = 10,
    Override = 50,
    SigKill = 100,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// White colour temperature in Kelvin.
    Temperature(u16),
    /// Direct RGB colour.
    Rgb { red: u16, green: u16, blue: u16 },
}

#[derive(Debug, Clone)]
pub struct LightingCommand {
    pub priority: Priority,
    pub state: bool,
    pub dimming: u8,
    /// Mutually exclusive colour choice. None leaves colour untouched.
    pub color: Option<ColorMode>,
    pub source: String,
}
```
