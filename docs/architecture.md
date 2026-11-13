# System Architecture: WiZ Controller

## 1. Core Principles
- Strict separation of policy (rules/recommendations) and mechanism (execution).
- No direct hardware calls from rule engines; everything flows through the Controller state machine.

## 2. Components
### A. Regulation Service (`src/udp.rs`)
- **Responsibility:** Pure hardware abstraction layer. Speaks only WiZ UDP JSON protocol on port 38899.
- **Rules:** "Dumb" executor. Receives commands strictly from the Controller. Contains zero business logic.

### B. Controller (`src/controller.rs`)
- **Responsibility:** Deterministic state machine and enforcer. 
- **Rules:** Evaluates incoming contracts based on priority (SIGKILL > Override > Standard) and enforces the desired state towards the Regulation Service.
- **Colour model:** Commands carry a single `Option<ColorMode>` value. RGB and colour temperature are mutually exclusive by contract, so the Controller never arbitrates between them.

### C. Recommendation Service (`src/recommendations.rs`)
- **Responsibility:** Business rule provider (e.g., fetching electricity prices, time-of-day logic, or AI recommendations).
- **Rules:** Emits `LightingCommand` structs to the Controller queue with intermediate priorities. Never communicates directly with the Regulation Service

### D. Timer Service / SIGKILL
- **Responsibility:** Hard safety net. 
- **Rules:** Emits highest-priority commands (e.g., hard shutdown at 23:30) directly to the controller queue (e.g. hard shutdown at 23:30) that cannot be overridden by standard rules.
