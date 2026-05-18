# CHANGELOG
**TinkerVault 42**

---

## [Unreleased] — Post-Audit Remediation

### To Fix (Stage 1)
- BUG-01: validate_date accepts impossible dates
- BUG-02: metadata not captured on wrap (timestamps, attributes, owner, document metadata, extended attributes)
- BUG-03: original file not deleted after wrap
- BUG-04: unique_path silently renames on collision instead of prompting
- WARN-01: duplicate unique_path in vault_format.rs and payload.rs
- WARN-03: clear_local_cache silently drops per-item errors
- WARN-04: julian_day_utc uses .unwrap()
- WARN-05: seed length check uses byte length not char count
- WARN-07: mode switch does not reset payloadPath
- WARN-08: default field values have no guard
- WARN-10: no release profile in Cargo.toml
- WARN-11: npm deps pinned to latest
- WARN-12: CLAUDE.md data flow diagram inaccurate

### To Add (Stage 1)
- Coordinate lookup helper button -> madtinkersworkshop.com/coords

### To Add (Stage 2)
- Minisign signing pipeline
- NSIS verification step (fail = no install)
- Full release package (manifest, sig, SHA256SUMS, VERIFY.txt)
- madtinkersworkshop.com/coords lookup page

---

## [42.1.0] — 2026-05-17

### Audit baseline established

**Architecture confirmed:**
- PolyGlyph95: 95x95 polyalphabetic substitution matrix, position-dependent, 5-digit hex per character
- Seed phrase split into 3 segments, interleaved at locks 2-3, 3-4, 4-5
- Lock chain: Coordinate, Date, Zulu time, Nautical star (scientific name), Combined hash
- GeoAstroLock42-Nautical57: real spherical astronomy, LST + hour angle + alt/az
- Argon2id Beast Mode: 1 GB RAM, 4 iterations, 1 thread
- TVLT42-1 binary vault format
- AES-256-GCM with AAD-authenticated header
- SHA256 payload integrity

**Coordinate resolution confirmed:**
- Path A: raw coordinates parsed directly
- Path B: place name normalized, checked against known-place table, SHA256 fallback if not found
- Stellar math runs against whatever coordinates are produced, real or pseudo

**Metadata capture confirmed NOT implemented:** payload stores name, bytes, SHA256 only. No timestamps, no attributes, no owner. Added as BUG-02.

**Removed in earlier development:**
- Python sidecar (replaced by pure Rust backend)
- geopy/Nominatim geocoding (replaced by SHA256 fallback, coord lookup helper planned)

---

## Notes

Version 42.1.0: the 42 is part of the name (meaning of life), not sequential versioning.
