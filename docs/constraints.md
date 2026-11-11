# Technical Constraints & Guardrails

## 1. Rule Precedence & Hierarchy
- **Supreme Authority:** This file (`constraints.md`) and `contracts.md` take absolute precedence over any coding style or pattern preferences. 
- If a conflict arises between architecture descriptions and type definitions, `contracts.md` is the single source of truth.

## 2. Language & Dependency Management
- **Language:** Modern Rust (Edition 2021 or newer).
- **Dependency Minimalism:** Zero unnecessary external crates. 
  - *Permitted core dependencies:* `serde`, `serde_json`, and `tokio` (only if async runtime is strictly required).
  - *Prohibited:* Heavy frameworks or abstraction layers that obscure low-level control.

## 3. Error Handling & Safety
- **Zero Panic Policy:** **No `unwrap()` or `expect()`** is permitted in production code paths. 
- All errors must be handled explicitly via `Result`, propagated gracefully, or logged with context without crashing the daemon.
- **Memory Safety:** Safe Rust only (`#![forbid(unsafe_code)]` unless proven strictly necessary for low-level systems interop with explicit justification).
  - By default, the codebase is strictly `#![forbid(unsafe_code)]`.
  - **Conditional exception** if low-level interop strictly requires `unsafe` it is only permitted if:
    1. The block is documented with a clear `// SAFETY...` comment.
    2. An entry is added to `docs/unsafe-why.md` explaining why safe Rust failed and which invariants are manually upheld.

## 4. Performance & Resource Footprint
- **Idle Efficiency:** The daemon must maintain **0.0% CPU usage** when idling (utilizing blocking sleeps or efficient async timers).
- **Memory Predictability:** Avoid unnecessary heap allocations in hot loops. Reuse buffers where appropriate for UDP payloads.

## 5. Network & Resilience
- **UDP Fault Tolerance:** Because UDP is unreliable by design, packet loss, timeouts, or network unreachable errors must be caught, logged as non-fatal warnings, and retried on the next reconciliation or control cycle.
