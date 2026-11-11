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

## 2. Internal Rust Contract
Every component that wants to control the lights must emit this exact struct to the Controller queue:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Standard = 10,
    Override = 50,
    SigKill = 100,
}

#[derive(Debug, Clone)]
pub struct LightingCommand {
    pub priority: Priority,
    pub state: bool,
    pub dimming: u8,
    pub color_temp: Option<u16>, //In Kelvin!
    pub rgb: Option<u16, u16, u16> // Color changes represented in RGB
    pub source: String,
}
```
