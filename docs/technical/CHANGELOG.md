# CHANGELOG
**TinkerVault 42**

---

## [Unreleased] — Stage 2: Build, Sign, Release Package

### To Do
- Generate MTW Minisign keypair, store private key on offline USB
- Complete sign_manifest.ps1 with Minisign
- Add Minisign verification as step one of NSIS installer (fail = no install)
- Build madtinkersworkshop.com/coords lookup page
- Verify bundle targets and icon set (Q-05)
- Full release build: npm run build:windows
- Smoke test on clean Windows machine
- Generate and sign release-manifest.json
- Produce full release package (exe, manifest, sig, SHA256SUMS, README, VERIFY.txt)
- Post to GitHub Releases and madtinkersworkshop.com

---

## [42.1.1] — 2026-05-18 — Stage 1 Audit Remediation
**Commit:** 0d44631

### Fixed
- **BUG-01:** validate_date now rejects impossible dates (Feb 30, Nov 31, etc.) with month-aware day validation and leap year check. Clear error messages include date and reason.
- **BUG-02:** Full metadata capture and restore implemented in payload.rs. FileMetadata struct with Option fields (backward-compatible). Captures and restores: created/modified/accessed timestamps (files and directories via FILE_FLAG_BACKUP_SEMANTICS), file attributes (Windows DWORD), Zone.Identifier (Mark of the Web), owner SID (best-effort restore without SeRestorePrivilege).
- **BUG-03:** Delete original prompt after successful wrap. Y: secure zero-wipe + delete. N: leaves file in place.
- **BUG-04:** unique_path consolidated from two files into utils::unique_path. Collision prompts user instead of silent rename. Pre-check before Beast Mode runs on wrap.
- **WARN-01/02:** unique_path consolidated, collision exhaustion returns error not original path.
- **WARN-03:** clear_local_cache per-item errors now surfaced instead of silently dropped.
- **WARN-04:** julian_day_utc .unwrap() replaced with Result, propagated through alt_az and forge_seed.
- **WARN-05:** Seed length check fixed to use chars().count() not len().
- **WARN-07:** payloadPath resets on mode switch.
- **WARN-08:** Wrap warns if place/date/skyTime match default field values.
- **WARN-10:** Cargo.toml release profile added: lto=true, codegen-units=1, strip=true.
- **WARN-11:** npm deps pinned to lock-file versions.
- **WARN-12:** CLAUDE.md data flow diagram updated to reflect actual execution order.

### Added
- Coordinate lookup helper button in App.jsx opens madtinkersworkshop.com/coords with URL-encoded place field value.

### Deferred
- BUG-02 remainder: document metadata (format-specific parsers), general NTFS ADS enumeration (BackupRead). Stage 3+ ticket.
- WARN-06: Large file streaming. Future.
- WARN-09: CSP policy. Deferred by design.

### Pre-existing (not touched)
- nautical57.rs:5 StarEntry::scientific field never read. Intentional, scientific name used elsewhere in pipeline.

---

## [42.1.0] — 2026-05-17 — Audit Baseline

### Audit findings
- Crypto pipeline, vault format, key derivation, IPC: PASS
- Date validation: FAIL (BUG-01)
- Metadata capture: not implemented (BUG-02)
- Original file not deleted after wrap (BUG-03)
- Silent rename on collision (BUG-04)
- Full WARN list documented in BUGS.md

### Architecture confirmed
- PolyGlyph95: 95x95 polyalphabetic substitution, position-dependent, 5-digit hex per character
- Seed phrase split into 3 segments, interleaved at locks 2-3, 3-4, 4-5
- Lock chain: Coordinate, Date, Zulu time, Nautical star (scientific name), Combined hash
- GeoAstroLock42-Nautical57: real spherical astronomy, LST + hour angle + alt/az
- Argon2id Beast Mode: 1 GB RAM, 4 iterations, 1 thread
- TVLT42-1 binary vault format, AES-256-GCM, AAD-authenticated header

### Removed in earlier development
- Python sidecar (replaced by pure Rust backend)
- geopy/Nominatim geocoding (replaced by SHA256 fallback + coord lookup helper)

---

## Notes

Version 42.1.x: the 42 is part of the name (meaning of life), not sequential versioning.
