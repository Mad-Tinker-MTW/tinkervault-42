# BUGS
**TinkerVault 42**
Last updated: 2026-05-18

---

## Open Bugs

### BUG-01 — FAIL — High
**File:** `geoastro.rs:91`
**Description:** `validate_date` only checks month (1-12) and day (1-31). Does not validate day against month or leap year. User entering `2023-02-30` or `2023-11-31` gets no error. The malformed date encodes into key material silently.
**Impact:** Vault created successfully. On unwrap, user enters the real intended date. Different key. Vault cannot be opened. No recovery path.
**Fix:** Reject impossible dates with a clear error. Validate day against month and leap year before encoding.
**Priority:** Fix before any v1.0 build.

---

### BUG-02 — FAIL — High
**File:** `payload.rs`
**Description:** Metadata not captured on wrap. `Payload::File` stores only `original_name`, `file_sha256`, `file_b64`. No timestamps, no attributes, no owner, no embedded document metadata, no extended attributes.
**Impact:** Unwrapped file gets restoration timestamp, not original. Attributes and owner are OS defaults. File is not forensically identical to original.
**Fix:** Extend payload struct to capture: created/modified/accessed timestamps, file attributes (Windows DWORD), owner (SID), document metadata (author, title, company, etc.), Zone.Identifier, NTFS alternate data streams. Restore all on unwrap. Use `Option` fields so existing vaults without metadata still unwrap correctly.
**Priority:** Stage 1.

---

### BUG-03 — FAIL — Medium
**File:** `lib.rs`, `App.jsx`
**Description:** After successful wrap, original file is not deleted. Plaintext sits next to the vault. Defeats the purpose of encrypting.
**Fix:** After successful wrap, prompt user: "Delete original? [Y/N]". Y: securely delete original. N: leave in place, user accepts responsibility. Default recommendation is Y.
**Priority:** Stage 1.

---

### BUG-04 — FAIL — Medium
**File:** `vault_format.rs:182`, `payload.rs`
**Description:** On unwrap, if the output filename already exists, `unique_path` silently renames to `.001`, `.002`, etc. User ends up with `secret.txt.001` with no explanation. Also connected to BUG-03: the original was never deleted so the filename collision is guaranteed.
**Fix:** Prompt user on collision rather than silent rename. "File already exists. Overwrite, rename, or cancel?"
**Priority:** Stage 1 (partially resolved by fixing BUG-03).

---

### BUG-05 — WARN — Low
**File:** `nautical57.rs`
**Description:** Catalog named Nautical-57 contains 58 entries. Zubenelgenubi is the extra entry beyond the standard 57 Nautical Almanac stars.
**Impact:** Naming discrepancy only. Both frontend and backend agree on 58 stars. No functional bug.
**Fix:** Decision required. Removing Zubenelgenubi would break any vault created with that star. If intentional, document it. If not, leave it and update the name.
**Priority:** Decision before any catalog change.

---

## Warnings (from audit 2026-05-17)

| ID | File | Description | Status |
|---|---|---|---|
| WARN-01 | vault_format.rs / payload.rs | Duplicate unique_path function | Open |
| WARN-02 | vault_format.rs:182 | unique_path silent overwrite after 10k collisions | Open (see BUG-04) |
| WARN-03 | lib.rs:92 | clear_local_cache silently drops per-item errors | Open |
| WARN-04 | geoastro.rs:119 | julian_day_utc uses .unwrap() | Open |
| WARN-05 | seedforge.rs:22 | Seed length check uses byte length not char count | Open |
| WARN-06 | payload.rs | No streaming, large files may exhaust RAM | Deferred |
| WARN-07 | App.jsx | Mode switch does not reset payloadPath | Open |
| WARN-08 | App.jsx | Default field values have no guard | Open |
| WARN-09 | tauri.conf.json | CSP is null | Deferred |
| WARN-10 | Cargo.toml | No release profile optimizations | Open |
| WARN-11 | package.json | Deps pinned to latest | Open |
| WARN-12 | CLAUDE.md | Data flow diagram execution order inaccurate | Open |

---

## Open Questions

| ID | Question | Decision Needed By |
|---|---|---|
| Q-01 | Is 58 stars intentional? Removing Zubenelgenubi breaks vaults using that star. | Before any catalog change |
| Q-02 | Reject impossible dates with error or clamp? Error is the correct answer. | Stage 1 before BUG-01 fix |
| Q-03 | Signing tool: Minisign confirmed. | Closed |
| Q-04 | Practical upper file size limit? | Stage 1 assessment |
| Q-05 | Icon set complete for all bundle targets? | Stage 2 build verification |
| Q-06 | Place name resolution: known-place table contents and coverage. SHA256 fallback behavior documented. Coord lookup helper planned. | Stage 1 |

---

## Closed Bugs

None yet.
