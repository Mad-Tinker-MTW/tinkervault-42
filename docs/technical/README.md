# TinkerVault 42 — README

MTW Security Division. Rust/Tauri. Windows-only. Deterministic multi-factor file encryption.

The key is a moment in time. Where you were, when, what was overhead, what you were thinking. Nobody else was there.

---

## Current Status

Post-audit. Stage 1 remediation in progress. Do not use in production until BUG-01 (date validation) and BUG-02 (metadata capture) are resolved.

See BUGS.md for full issue list. See ROADMAP.md for schedule.

---

## Quick Start

**Dev:**
```powershell
npm run dev:tauri
```

**Build:**
```powershell
npm run build:windows
```

**Rust only:**
```powershell
cd src-tauri
cargo check
cargo build --release
cargo test
```

---

## Five Lock Factors

1. Coordinate (place name or raw lat,lon)
2. Date (YYYY-MM-DD)
3. Zulu time (UTC)
4. Nautical star (common name, converted to scientific internally)
5. Seed phrase (1-3 words, up to 95 chars each, run through PolyGlyph95)

All five must match exactly. No recovery path.

---

## What Makes It Different

- **PolyGlyph95:** 95x95 polyalphabetic substitution. Same character at different positions maps to different glyphs. `Apple`, `aPpLe`, and `@pp13` all produce different key material.
- **Seed interleaving:** Seed segments distributed across the lock chain. Two locks must be broken before the first segment is reachable.
- **Real astronomy:** The app calculates where your star actually was in the sky at your coordinates and time. The key is anchored to a real moment.
- **Forensic restore:** Unwrapped file is identical to the original. All metadata preserved. (BUG-02: not yet implemented, Stage 1 target.)
- **Argon2id Beast Mode:** 1 GB RAM, 4 iterations. Brute force is not practical.

---

## Documentation

**Technical:**
- SPEC.md -- full pipeline, vault format, module reference
- CHANGELOG.md -- version history and audit findings
- ROADMAP.md -- stage plan
- BUGS.md -- open bugs and open questions
- VISION.md -- design philosophy and Security Division product stack

**PMP:**
- TV42-PMD-001 Charter
- TV42-PMD-002 Scope
- TV42-PMD-003 WBS
- TV42-PMD-004 Schedule
- TV42-PMD-005 Risk Register
- TV42-PMD-006 Stakeholder Register

---

## Hard Constraints

- Argon2id Beast Mode: 1 GB RAM required. Non-negotiable.
- TVLT42-1 header layout is frozen. Do not change magic bytes or structure.
- All 5 lock factors must match exactly. Order and casing matter.
- Place resolution is offline. No geocoding APIs in the app.

---

## Operator

Mad Tinker (Francisco De La Paz)
MTW Security Division / 4Kings Enterprises, Altus OK
