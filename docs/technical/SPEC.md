# SPEC
**TinkerVault 42**
Version: 1.1
Date: 2026-05-18

---

## Overview

TinkerVault 42 is a Rust/Tauri desktop application for deterministic multi-factor file and folder encryption. Windows-only. No network. No key files. No password manager. The encryption key is derived entirely from five lock factors the operator supplies at runtime.

Full metadata preservation on wrap and restore. The unwrapped file is forensically identical to the original.

---

## Five Lock Factors

| # | Factor | Input | Internal Processing |
|---|---|---|---|
| 1 | Coordinate | Place name or raw lat,lon | Place name maps to coordinates via system lookup, frozen. Raw coordinates accepted directly. Case insensitive. |
| 2 | Date | YYYY-MM-DD | Validated including day-of-month against month and leap year |
| 3 | Zulu time | HH:MM or HHMM (UTC) | Normalized to UTC, eliminates timezone ambiguity |
| 4 | Nautical star | Common name | Converted to scientific designation internally before hashing |
| 5 | Seed phrase | 1-3 words, up to 95 characters each including spaces | Run through PolyGlyph95, split into three segments, interleaved into lock chain |

All five must match exactly on wrap and unwrap. No recovery path.

---

## PolyGlyph95

A 95x95 polyalphabetic substitution matrix. 95 rows, each row is a unique randomized permutation of all 95 printable ASCII characters.

Character substitution is position-dependent. The same character at a different position in the input maps to a completely different glyph. This means:

```
Apple  -> different glyph output
aPpLe  -> different glyph output
@pp13  -> different glyph output
```

Each character maps to a 5-digit hex value. A 30-character seed word produces 150 characters of hex output before hashing.

The 95x95 matrix generation algorithm and seed segment distribution method are MTW internal. Not for public documentation.

---

## Lock Chain and Seed Interleaving

The seed phrase is split into three segments after PolyGlyph95 processing. Segments are interleaved into the lock chain, not processed as a block:

```
Lock 1: Coordinate
Lock 2: Date
  -> Seed segment 1 inserted here
Lock 3: Zulu time
  -> Seed segment 2 inserted here
Lock 4: Nautical star (scientific name)
  -> Seed segment 3 inserted here
Lock 5: Combined final hash -> Argon2id -> AES-256-GCM
```

Two locks must be broken before the first seed segment is reachable. Each subsequent segment is gated behind another lock. The seed is never exposed as a unit.

The exact interleave positions are MTW internal. Not for public documentation.

---

## Stellar Position (GeoAstroLock42-Nautical57)

The nautical star common name converts to its scientific designation internally. RA/Dec pulled from the fixed 58-entry Nautical57 catalog.

Using frozen coordinates and Zulu time, the app computes:

- Local Sidereal Time: `LST = GMST + longitude/15.0`
- Hour angle: angular distance between the star and the observer's meridian
- Altitude and azimuth via spherical trig: hour angle + declination + latitude

The result is the real position of that star in the sky at that exact location and moment. Only astronomically accurate when real coordinates are used.

---

## Coordinate Resolution

Two input paths, same frozen output:

**Path A: Raw coordinates**
User enters `40.8517,-73.9527`. Parsed directly.

**Path B: Place name**
User enters a place name. System normalizes (lowercase, collapse whitespace), checks internal known-place table. If found, real coordinates returned and frozen. If not found, SHA256 deterministic fallback produces a pseudo-coordinate (stable and reproducible but not geographically real).

**Coordinate lookup helper:**
[Look up] button opens `madtinkersworkshop.com/coords?q=place+name` in the default browser. Page returns lat/lon with copy buttons. App never touches the network. User pastes result into the coordinate field.

Case does not matter. Ambiguous place names prompt for clarification with suggested options.

---

## Key Derivation Pipeline

```
Seed phrase
  -> PolyGlyph95 (95x95 matrix, position-dependent, 5-digit hex per char)
  -> Split into segments A, B, C

Lock 1: Coordinate (frozen lat/lon)
Lock 2: Date (validated YYYY-MM-DD)
  -> Segment A
Lock 3: Zulu time (UTC)
  -> Segment B
Lock 4: Star scientific name + altitude/azimuth at location/time
  -> Segment C
Lock 5: Combined material
  -> Argon2id Beast Mode (1 GB RAM, 4 iterations, 1 thread) -> 32-byte key
  -> AES-256-GCM encrypt payload
```

---

## Argon2id Beast Mode

```
Memory:      1,048,576 KiB (1 GB)
Iterations:  4
Parallelism: 1
Salt:        32 bytes OsRng
Output:      32-byte key
```

Non-negotiable. Parameters stored in the TVLT42-1 vault header.

---

## Vault Format (TVLT42-1)

Binary header (unencrypted, authenticated as AAD):

```
magic[8]         = b"TVLT42\x00\x01"
cipher_id[1]     = 0x01 (AES-256-GCM)
mem_kib[4]       = 1048576 (little-endian u32)
iterations[4]    = 4 (little-endian u32)
parallelism[4]   = 1 (little-endian u32)
salt_len[1]      = 32
nonce_len[1]     = 12
salt[32]
nonce[12]
```

Followed by AES-256-GCM ciphertext of JSON payload.

Magic bytes and header layout are frozen. Any change breaks all existing vaults.

---

## Payload Metadata Capture

On wrap, the following is captured and stored in the vault payload:

**File system metadata:** created/modified/accessed timestamps, file attributes (hidden, readonly, system, archive), file size, owner (Windows SID)

**Document/embedded metadata:** author, last modified by, created by application, title, subject, keywords, company, revision number, comments

**Extended attributes:** Zone.Identifier (Mark of the Web), NTFS alternate data streams

On unwrap, all captured metadata is restored. The file is forensically identical to the original.

This metadata store is the foundation for a future standalone MTW Anonymity Tool that strips or spoofs selected fields on output.

---

## Wrap Behavior

After successful wrap:

1. `.TinkerVault` file produced
2. Prompt: **Delete original? [Y/N]**
3. Y: original securely deleted. Only vault remains.
4. N: original left in place. User's responsibility.

On unwrap, collision handling prompts the user rather than silently renaming.

---

## Module Reference

| Module | Job |
|---|---|
| `main.rs` | Entry point, suppresses console in release |
| `lib.rs` | Tauri IPC: wrap_payload, unwrap_vault, clear_local_cache |
| `vault_format.rs` | TVLT42-1 header, encrypt/decrypt dispatch |
| `crypto.rs` | Argon2id KDF, AES-256-GCM |
| `seedforge.rs` | PolyGlyph95, seed segmentation, lock chain assembly |
| `geoastro.rs` | Coordinate resolution, date/time normalization, alt/az math |
| `nautical57.rs` | 58-entry fixed star catalog with RA/Dec |
| `payload.rs` | File/folder serialize, full metadata capture/restore, SHA256 integrity |
| `errors.rs` | VaultError enum, Result alias |

---

## Build Requirements

- Rust stable toolchain
- Node.js v20+, npm
- Tauri CLI v2
- Windows with WebView2 runtime
- NSIS (for installer)
- Minimum 2 GB available RAM at runtime
