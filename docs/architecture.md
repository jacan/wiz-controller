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
- **Reconciliation cadence:** The Controller retries failed reconciliations every 250 ms and resends the desired state every second to tolerate silently dropped UDP datagrams.
- **Failure diagnostics:** Failed reconciliations are logged as non-fatal warnings and retained in a bounded in-memory ring buffer (default 64 entries). The buffer is diagnostic only and resets on restart.

### C. Recommendation Service (`src/recommendations.rs`)
- **Responsibility:** Business rule provider (e.g., fetching electricity prices, time-of-day logic, or AI recommendations).
- **Rules:** Emits `LightingCommand` structs to the Controller queue with intermediate priorities. Never communicates directly with the Regulation Service

### D. Timer Service / SIGKILL
- **Responsibility:** Hard safety net. 
- **Rules:** Emits highest-priority commands (e.g., hard shutdown at 23:30) directly to the controller queue (e.g. hard shutdown at 23:30) that cannot be overridden by standard rules.
