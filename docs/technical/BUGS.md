# BUGS
**TinkerVault 42**
Last updated: 2026-05-18

---

## Open Bugs

### BUG-05 — WARN — Low
**File:** `nautical57.rs`
**Description:** Catalog named Nautical-57 contains 58 entries. Zubenelgenubi is the extra entry beyond the standard 57 Nautical Almanac stars.
**Impact:** Naming discrepancy only. Both frontend and backend agree on 58 stars. No functional bug.
**Fix:** Decision required. Removing Zubenelgenubi would break any vault created with that star. If intentional, document it. If not, leave it and update the name.
**Priority:** Decision before any catalog change.

---

## Warnings (Open)

| ID | File | Description | Status |
|---|---|---|---|
| WARN-06 | payload.rs | No streaming, large files may exhaust RAM | Deferred |
| WARN-09 | tauri.conf.json | CSP is null | Deferred |

---

## Open Questions

| ID | Question | Decision Needed By |
|---|---|---|
| Q-01 | Is 58 stars intentional? Removing Zubenelgenubi breaks vaults using that star. | Before any catalog change |
| Q-04 | Practical upper file size limit? | Stage 1 assessment |
| Q-05 | Icon set complete for all bundle targets? | Stage 2 build verification |
| Q-06 | Place name resolution: known-place table contents and coverage. SHA256 fallback behavior documented. Coord lookup helper planned. | Stage 1 |

---

## Closed Bugs

### BUG-01 — FIXED — 2026-05-17
**File:** `geoastro.rs:91`
**Was:** `validate_date` only checked month (1-12) and day (1-31). Accepted impossible dates like 2023-02-30 and 2023-11-31 silently, encoding them into key material.
**Fix:** Full month-aware validation with leap year logic. Errors return the full date and reason, e.g. `"Invalid date 2023-02-30: month 2 has at most 28 days."` `_y` parameter promoted to `y` and used in leap year calculation.

---

### BUG-02 — PARTIAL — 2026-05-18
**File:** `payload.rs`
**Was:** No metadata captured on wrap. Restored files had restoration timestamp, OS-default attributes, no Zone.Identifier.
**Fixed:** `FileMetadata` struct added with `Option<>` fields (backward-compatible). Captured and restored: `created_secs`, `modified_secs`, `accessed_secs` (via `CreateFileW` + `SetFileTime` + `FILE_FLAG_BACKUP_SEMANTICS`, works for both files and directories), `file_attributes` (Windows DWORD via `MetadataExt` + `SetFileAttributesW`), `zone_identifier` (Zone.Identifier ADS read/write via stream path), `owner_sid` (SID string via `GetSecurityInfo` / `SetSecurityInfo` + `ConvertSidToStringSidW` / `ConvertStringSidToSidW`; restore is best-effort, silently skipped without `SeRestorePrivilege`). Attributes applied last so READONLY is set after all other metadata.

**Remaining TODO (separate tickets when scoped):**
- Document metadata (author, title, company): needs format-specific parsers (OOXML, PDF). Out of scope for Stage 1.
- General NTFS ADS enumeration: needs `BackupRead` or `NtQueryEaFile`. Only Zone.Identifier handled explicitly.

---

### BUG-03 — FIXED — 2026-05-18
**File:** `lib.rs`, `App.jsx`
**Was:** After successful wrap, original plaintext file remained beside the vault with no prompt.
**Fix:** After a successful wrap, frontend shows an `ask` dialog: "Wrapped successfully. Delete original? This cannot be undone." Yes invokes `delete_original` Tauri command. `delete_original` zero-wipes file content before deletion (single-pass). Folder wraps wipe all files recursively via `WalkDir` before `remove_dir_all`. SSD caveat: single-pass zero wipe does not guarantee erasure on flash storage with wear leveling — noted by design.

---

### BUG-04 — FIXED — 2026-05-18
**File:** `vault_format.rs`, `payload.rs`, `lib.rs`, `App.jsx`
**Was:** `unique_path` silently renamed collision outputs to `.001`, `.002`, etc. Duplicate function in both `vault_format.rs` and `payload.rs`. Collision detection happened after Beast Mode on unwrap.

**Fix:**
- Consolidated `unique_path` into `utils.rs` with `overwrite: bool` parameter. Errors on collision with `"COLLISION:<path>"` prefix instead of silent rename. Old auto-rename loop removed.
- `VaultRequest` gains `#[serde(default)] overwrite: bool`.
- Wrap pre-check: frontend calls `check_wrap_collision` before Beast Mode. If output exists, shows dialog immediately. User picks Overwrite or Cancel. No double Beast Mode for wrap.
- Unwrap: COLLISION error caught in frontend after decrypt. User picks Overwrite or Cancel. Re-invoke with `overwrite: true` if confirmed (Beast Mode runs again — unavoidable without decrypting twice, acceptable since collision on unwrap is rare if BUG-03 workflow is followed).

---

## Warnings (Closed)

| ID | File | Description | Closed |
|---|---|---|---|
| WARN-01 | vault_format.rs / payload.rs | Duplicate unique_path function | 2026-05-18 — consolidated into utils.rs |
| WARN-02 | vault_format.rs:182 | unique_path silent overwrite after 10k collisions | 2026-05-18 — replaced with COLLISION error |
| WARN-03 | lib.rs:92 | clear_local_cache silently drops per-item errors | 2026-05-18 — failures reported in return string |
| WARN-04 | geoastro.rs:119 | julian_day_utc uses .unwrap() | 2026-05-18 — returns Result<f64>, ? propagated through alt_az and forge_seed |
| WARN-05 | seedforge.rs:22 | Seed length check uses byte length not char count | 2026-05-18 — changed to chars().count() |
| WARN-07 | App.jsx | Mode switch does not reset payloadPath | 2026-05-18 — mode buttons now clear payloadPath and payloadType |
| WARN-08 | App.jsx | Default field values have no guard | 2026-05-18 — ask confirmation on wrap if place/date/skyTime match defaults |
| WARN-10 | Cargo.toml | No release profile optimizations | 2026-05-18 — lto=true, codegen-units=1, strip=true added |
| WARN-11 | package.json | Deps pinned to latest | 2026-05-18 — react 19.2.6, react-dom 19.2.6, vite 8.0.13, lucide-react 1.16.0, framer-motion 12.38.0, @vitejs/plugin-react 6.0.2 |
| WARN-12 | CLAUDE.md | Data flow diagram execution order inaccurate | 2026-05-18 — diagram updated: payload built before key derivation, geoastro called inside seedforge |

---

## Closed Questions

| ID | Question | Resolution |
|---|---|---|
| Q-02 | Reject impossible dates with error or clamp? | Error. Implemented in BUG-01 fix. |
| Q-03 | Signing tool? | Minisign confirmed. |
