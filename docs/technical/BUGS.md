# BUGS
**TinkerVault 42**
Last updated: 2026-05-18

---

## Open Bugs

### BUG-02 (PARTIAL) — Document metadata and general ADS
**File:** `payload.rs`
**Description:** Document-embedded metadata (author, title, company, revision, etc.) and general NTFS ADS enumeration not captured.
**Status:** Deferred. Requires format-specific parsers (OOXML, PDF) for document metadata and BackupRead/NtQueryEaFile for general ADS. Out of scope for Stage 1.
**Priority:** Stage 3+ separate ticket.

---

## Closed Bugs

### BUG-01 — CLOSED — 2026-05-18
**Commit:** 0d44631
**File:** `geoastro.rs:91`
**Was:** validate_date accepted impossible dates (Feb 30, Nov 31, etc.). Invalid date encoded into key material silently, creating unrecoverable vault.
**Fixed:** Month-aware day validation with leap year check. Error messages include full date and reason:
- "Invalid date 2023-02-30: month 2 has at most 28 days."
- "Invalid date 2024-02-30: month 2 has at most 29 days." (leap year)
- "Invalid date 2023-11-31: month 11 has at most 30 days."

---

### BUG-02 (MAJOR PORTION) — CLOSED — 2026-05-18
**Commit:** 0d44631
**File:** `payload.rs`
**Was:** No metadata captured. Restored files had restoration timestamp, OS-default attributes, no provenance.
**Fixed:** `FileMetadata` struct added with `Option<>` fields (backward-compatible with existing vaults). Captured and restored:
- created/modified/accessed timestamps via `CreateFileW` + `SetFileTime` + `FILE_FLAG_BACKUP_SEMANTICS` (works for files and directories)
- file attributes (Windows DWORD) via `MetadataExt` + `SetFileAttributesW`
- Zone.Identifier (Mark of the Web) via ADS stream path read/write
- owner SID string via `GetSecurityInfo` / `SetSecurityInfo` + `ConvertSidToStringSidW` / `ConvertStringSidToSidW` (restore is best-effort, silently skipped without SeRestorePrivilege)
- Attributes applied last so READONLY is set after timestamps and Zone.Identifier are written

**Remaining deferred:** document metadata (format-specific parsers), general NTFS ADS enumeration (BackupRead).

---

### BUG-03 — CLOSED — 2026-05-18
**Commit:** 0d44631
**File:** `lib.rs`, `App.jsx`
**Was:** Original file left in plaintext next to vault after wrap.
**Fixed:** Ask dialog after successful wrap. Y: secure zero-wipe then delete via `delete_original` command. N: leaves file, user accepts responsibility.

---

### BUG-04 — CLOSED — 2026-05-18
**Commit:** 0d44631
**File:** `vault_format.rs`, `payload.rs`
**Was:** `unique_path` duplicated in two files. Collision silently renamed to `.001`.
**Fixed:** Consolidated into `utils::unique_path`. COLLISION error protocol: pre-check for wrap before Beast Mode runs, re-invoke with `overwrite:true` for unwrap. User prompted on collision instead of silent rename.

---

## Warnings — Closed (Stage 1, commit 0d44631)

| ID | Was | Fixed |
|---|---|---|
| WARN-01/02 | Duplicate unique_path, silent overwrite on collision | Consolidated to utils::unique_path, error on collision |
| WARN-03 | clear_local_cache silently dropped per-item errors | Errors now surfaced |
| WARN-04 | julian_day_utc used .unwrap() | Returns Result, propagated through alt_az and forge_seed |
| WARN-05 | Seed length check used byte length | Fixed to chars().count() |
| WARN-07 | Mode switch did not reset payloadPath | Fixed |
| WARN-08 | Default field values had no guard | Warns on wrap if place/date/skyTime match defaults |
| WARN-10 | No release profile in Cargo.toml | Added lto, codegen-units, strip |
| WARN-11 | npm deps pinned to latest | Pinned to lock-file versions |
| WARN-12 | CLAUDE.md data flow diagram inaccurate | Updated |

## Warnings — Deferred

| ID | Description | Status |
|---|---|---|
| WARN-06 | No streaming, large files may exhaust RAM | Deferred to future |
| WARN-09 | CSP null in tauri.conf.json | Deferred by design for now |

---

## Open Questions

| ID | Question | Status |
|---|---|---|
| Q-01 | Is 58 stars intentional? | Open |
| Q-04 | Practical upper file size limit? | Open |
| Q-05 | Icon set complete for all bundle targets? | Verify in Stage 2 |
| Q-06 | Known-place table coverage | Open |
