# TinkerVault 42 — Audit Report
**Date:** 2026-05-17
**Auditor:** Claude Code (Sonnet 4.6)
**Scope:** All source files in `src-tauri/src/`, `src/`, build configs

---

## Summary

The core is solid. Encryption, key derivation, vault format, and IPC are correct. No logic errors in the crypto pipeline. One legitimate bug (date validation), one naming discrepancy (star count), and a cluster of warnings around missing release profile optimizations, pinned-to-latest npm deps, and two duplicated utility functions. No security vulnerabilities found in the vault code.

| Area | Status |
|---|---|
| Crypto pipeline (Argon2id + AES-256-GCM) | PASS |
| Vault format (TVLT42-1) | PASS |
| Key derivation data flow | PASS |
| IPC field mapping (frontend <-> Rust) | PASS |
| Frontend 5-factor validation | PASS |
| Date validation | FAIL |
| Star catalog count vs. name | WARN |
| Duplicate utility functions | WARN |
| Release profile config | WARN |
| npm dependency pinning | WARN |
| CSP config | WARN |
| Signing | TODO |
| Test coverage | TODO |

---

## Module Audit

### `main.rs` — PASS
Thin entry point. Calls `lib.rs::run()`. `windows_subsystem = "windows"` suppresses the console window in release builds. Nothing to flag.

### `errors.rs` — PASS
All five variants (`Io`, `Json`, `Base64`, `Crypto`, `Message`) are used. `From` impls for `std::io::Error`, `serde_json::Error`, and `base64::DecodeError` are all exercised. Clean.

### `crypto.rs` — PASS
Beast Mode constants confirmed: `ARGON2_MEMORY_KIB = 1_048_576` (1 GB), `ARGON2_ITERATIONS = 4`, `ARGON2_PARALLELISM = 1`. AES-256-GCM with AAD-authenticated encryption via `aes_gcm::aead::Payload`. `OsRng` for random salt and nonce. All matches the hard constraints in CLAUDE.md. `decrypt` returns `VaultError::Crypto` on authentication failure, which correctly obscures whether failure was wrong key vs. corrupted data.

### `vault_format.rs` — PASS / WARN

**PASS:** TVLT42-1 format is correct. Magic bytes `b"TVLT42\x00\x01"` (8 bytes). Header layout in code: `magic[8] | cipher_id[1] | mem_kib[4] | iterations[4] | parallelism[4] | salt_len[1] | nonce_len[1]` = 23-byte fixed header, then `salt[32]` and `nonce[12]`. AAD covers the complete header including salt and nonce, which is correct — the header is authenticated alongside the ciphertext.

**WARN:** Two functions are identical between this file and `payload.rs`:

```
vault_format.rs:unique_path  (line 163)
payload.rs:unique_path       (line 133)
```

Same logic, same 10,000-iteration limit, same fallback. DRY violation. Not a bug today, but a divergence risk.

**WARN:** `unique_path` returns the original (potentially existing) path if 10,000 collisions are found (line 182-184). This means the 10,001st vault of the same name would silently overwrite. Unlikely in practice but the fallback is wrong — should error, not overwrite.

**WARN:** CLAUDE.md header spec lists `argon2_params[12]` but omits the `salt_len[1]` and `nonce_len[1]` bytes. The actual fixed header is 23 bytes, not 21. Minor doc inaccuracy.

### `seedforge.rs` — PASS / WARN

**PASS:** PolyGlyph95-42 pipeline is deterministic and complete. `forge_seed` calls `resolve_place_lock`, `normalize_date`, `normalize_sky_time`, `find_star`, then `polyglyph95` -> `split_three` -> `glyph5` x3 -> `alt_az`. The resulting `material` string encodes all five factors. Determinism is guaranteed by SHA256-seeded Fisher-Yates shuffle in `deterministic_shuffle`.

**WARN:** Minimum seed check at line 22 uses `seed.len()` (byte length), not `seed.chars().count()` (char count). For the ASCII-only PRINTABLE95 charset this is equivalent, but it's a subtle inconsistency. A multi-byte UTF-8 character would pass the byte-length check of 3 and then immediately fail in `polyglyph95` with "Unsupported seed character." The error is clear but the length check semantics are slightly off.

**WARN:** `split_three` at line 128 performs integer division. With a 3-character seed: `a=1`, `b=2`, producing segments of length 1, 1, 1. All non-empty. Minimum is enforced. Verified safe at the 3-character boundary, but it's worth knowing — if the minimum were ever raised or the split logic were changed, this boundary needs re-checking.

### `geoastro.rs` — PASS / FAIL

**PASS:** `resolve_place_lock` handles both coordinate input (lat,lon parse) and text-based deterministic fallback (SHA256). Offline. No external API. Correct.

**PASS:** `normalize_sky_time` handles `HHMM` and `HH:MM`. Validates 00:00–23:59 correctly.

**PASS:** `parse_coords` accepts comma, semicolon, or space separator. Range validates lat (-90..90) and lon (-180..180). Correct.

**FAIL:** `validate_date` (line 91) only checks `m in 1..=12` and `d in 1..=31`. It does not validate:
- Day 31 for 30-day months (April, June, September, November)
- Day 29-31 for February
- Leap year status for February 29

A user entering `2023-02-30` or `2023-11-31` gets no error. The malformed date encodes into the key material. The vault is created but the user cannot reopen it because they would (correctly) enter the real date the second time and get a different key. **This creates orphaned vaults with no recovery path.**

**WARN:** `julian_day_utc` (line 118) calls `.unwrap()` on string parses of date and sky_time. These are safe only if called after normalization. It is always called after normalization in the current code path, but it is a hidden panic if the function is ever called directly with raw input.

### `nautical57.rs` — WARN

**WARN:** The catalog is named `Nautical-57` and the project is called `GeoAstroLock42-Nautical57`, implying 57 stars. The actual `STARS` array contains **58 entries** (Acamar through Zubenelgenubi). This is not a functional bug — both frontend and backend agree on the same 58 stars — but the name-to-count mismatch should be intentional or corrected. The standard Nautical Almanac selected stars are 57; Zubenelgenubi makes 58.

`find_star` normalizes by stripping all non-alphanumeric characters and lowercasing before matching. This means "Al Na'ir" and "Alnair" both resolve to the same star. Correct.

### `payload.rs` — PASS / WARN

**PASS:** `build_payload` reads files and folders, computes SHA256, base64-encodes. `restore_payload` verifies SHA256 before writing each file.

**PASS:** `validate_relative_path` checks for `..`, leading `/`, leading `\`, and `:` (blocks drive-letter paths on Windows). Path traversal is blocked. Backslash-to-forward-slash normalization happens during build (line 58), so entries always use forward slashes.

**WARN:** Duplicate `unique_path` function. See `vault_format.rs` note above.

**WARN:** No streaming. Entire file is read into memory as a byte array, base64-encoded (1.33x size inflation), then JSON-serialized. For large files, this combines with the 1 GB Argon2 allocation. A 500 MB file would require approximately 500 MB + 667 MB (base64) + JSON overhead + 1 GB Argon2 = ~2.2 GB peak RAM. Not a bug, but worth knowing for expected use cases.

### `lib.rs` — PASS / WARN

**PASS:** `wrap_payload` and `unwrap_vault` correctly bridge frontend to `vault_format.rs`. `clear_local_cache` sweeps `LOCALAPPDATA`, `APPDATA`, and `TEMP` for TinkerVault paths.

**WARN:** `clear_local_cache` silently drops per-item removal errors (line 92: `if result.is_ok()`). The return value is a success count only. A partial failure (e.g., file locked) is invisible to the user.

---

## Data Flow Verification

CLAUDE.md describes the flow as five sequential stages. The actual code is slightly different:

| CLAUDE.md stage | Actual call order in `wrap_path` | Match? |
|---|---|---|
| seedforge.rs (PolyGlyph95-42) | `forge_seed()` called first | PASS |
| geoastro.rs (coordinate + stellar) | Called *inside* `forge_seed`, not as a separate step | Minor |
| crypto.rs (Argon2id key) | `derive_key()` called after payload is built | PASS |
| vault_format.rs (AES-256-GCM) | `encrypt()` called after key derivation | PASS |
| payload.rs (serialize + SHA256) | `build_payload()` called *before* key derivation | Order differs |

The CLAUDE.md diagram implies payload serialization happens after crypto. In reality, `build_payload` runs before Argon2id to avoid holding 1 GB RAM unnecessarily while doing file I/O. The result is identical. The doc is logically accurate but operationally sequenced differently. Not a bug, but worth updating in CLAUDE.md.

---

## Frontend Audit

### Factor validation — PASS
`validateInputs` (line 122) checks all six required fields: `payloadPath`, `seed` (trimmed), `place` (trimmed), `date` (trimmed), `skyTime` (trimmed), `star` (trimmed). All five lock factors are required before any Tauri command is invoked.

### IPC field mapping — PASS
Frontend `request` object keys (`payloadPath`, `seed`, `place`, `date`, `skyTime`, `star`) map correctly to Rust `VaultRequest` via `#[serde(rename_all = "camelCase")]`.

### Star list — PASS (with WARN)
Frontend `stars` array (App.jsx line 25-34) matches the 58-entry `STARS` catalog in `nautical57.rs`. No mismatches. All frontend star names resolve to valid backend entries via `find_star`'s normalized matching.

### Drag-and-drop — PASS
`getCurrentWebview().onDragDropEvent` wired in `useEffect`. Cleanup via `unlisten` returned from the effect. No memory leak.

### Busy state — PASS
`setBusy(true)` before, `setBusy(false)` in `finally`. Buttons disabled during operation. Correct.

### Mode switch — WARN
Switching between Wrap and Unwrap modes does not reset `payloadPath`. A non-vault file selected in Wrap mode remains visible when switching to Unwrap. User would then try to unwrap a non-vault file and get a Rust error. Cosmetically confusing.

### Default field values — WARN
`place` defaults to `"40.814870,-73.888250"`, `date` to `"1985-09-20"`, `skyTime` to `"21:00"`. These are realistic-looking values. A user who does not clear the defaults before wrapping would produce a vault keyed with partially default factors. No guard against this.

### CSP — WARN (see Build Config)

---

## Build Config

### `tauri.conf.json` — WARN
`"csp": null` disables Content Security Policy entirely. For an offline internal tool that loads no external content, the practical risk is low. Tauri's recommended default is a strict CSP. Worth setting even a minimal policy.

Bundle `targets: "all"` generates all Tauri-supported bundle types. The `icon` array lists only `icons/icon.ico`. If NSIS or other targets require additional PNG icon sizes, the build may emit warnings or fail. Verify bundle output covers all required targets.

### `Cargo.toml` — WARN
No `[profile.release]` section. Default release profile is used. For a production binary:

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
```

These reduce binary size and improve performance at the cost of longer compile time. Not affecting correctness.

`crate-type = ["staticlib", "cdylib", "rlib"]` includes `staticlib`, which is unlikely to be needed for a Tauri binary. Minor.

### `package.json` — WARN
React, Vite, Framer Motion, and Lucide React are pinned to `latest`. This means `npm install` on a fresh machine may pick up breaking major versions. All Tauri-related packages are properly pinned to `^2.0.0`. Non-Tauri deps should be pinned to current versions for reproducible builds.

### `build_windows.ps1` — PASS
Checks for `icon.ico` before building. Fails fast with `$ErrorActionPreference = "Stop"`. Clean.

### `sign_manifest.ps1` — TODO
Creates a `release-manifest.json` with SHA256. Signing step is explicitly a placeholder — the script prints instructions but does not sign. The README mentions a planned release verification badge. Neither is implemented.

---

## Bugs Found

| ID | File | Description | Severity |
|---|---|---|---|
| BUG-01 | `geoastro.rs:91` | `validate_date` accepts impossible dates (Feb 30, Nov 31, etc.). Invalid date silently encodes into key material, creating an unrecoverable vault. | High |
| BUG-02 | `nautical57.rs` | Catalog named Nautical-57 contains 58 entries. Naming discrepancy. | Low |

---

## Warnings

| ID | File | Description |
|---|---|---|
| WARN-01 | `vault_format.rs:163` / `payload.rs:133` | Duplicate `unique_path` function. Two files, same logic. |
| WARN-02 | `vault_format.rs:182` | `unique_path` returns original path (potential overwrite) after 10,000 collisions. Should error. |
| WARN-03 | `lib.rs:92` | `clear_local_cache` silently drops per-item removal errors. User sees count only, no failure detail. |
| WARN-04 | `geoastro.rs:119` | `julian_day_utc` uses `.unwrap()` on parse. Safe today (only called post-normalization), but hidden panic if called directly. |
| WARN-05 | `seedforge.rs:22` | Minimum seed length check uses `seed.len()` (byte length) not `seed.chars().count()`. Harmless for ASCII-only PRINTABLE95 but semantically inconsistent. |
| WARN-06 | `payload.rs` | No streaming. Large files (500MB+) combined with 1GB Argon2 peak may exceed available RAM. |
| WARN-07 | `App.jsx:60-70` | Mode switch does not reset `payloadPath`. A wrap-mode file path carries over to unwrap. |
| WARN-08 | `App.jsx:65-68` | Default field values for place, date, and sky time are realistic-looking. No guard against accidental use. |
| WARN-09 | `tauri.conf.json` | CSP is null. Acceptable for offline internal tool but not best practice. |
| WARN-10 | `Cargo.toml` | No `[profile.release]` optimizations (lto, codegen-units, strip). |
| WARN-11 | `package.json` | React, Vite, and UI library deps pinned to `latest`. Non-reproducible across installs. |
| WARN-12 | `CLAUDE.md` | Data flow diagram shows payload serialization after crypto. Actual order is reversed (payload built first, then key derived). |

---

## Open Questions

| ID | Description |
|---|---|
| Q-01 | **Star count:** Is 58 in the catalog intentional, or should Zubenelgenubi be removed to match the Nautical-57 name? The key material uses the star ID, so a future removal would break vaults that used that star. Decision needed before any change. |
| Q-02 | **Date validation (BUG-01):** Should impossible dates (Feb 30, etc.) be rejected with an error, or silently clamped? Rejecting with an error is the safe call. |
| Q-03 | **Signing:** What signing tool is planned for `sign_manifest.ps1`? Minisign, OpenSSL, age-plugin-yubikey, or Sigstore are all options mentioned in the script. |
| Q-04 | **Large file support:** Is there a practical upper limit on file size for wrapping? If files over ~1 GB are expected, streaming architecture would be needed. |
| Q-05 | **Icon assets:** Does the current icon set satisfy all bundle targets ("all")? Verify the NSIS and MSI bundles produce without icon-related warnings. |
