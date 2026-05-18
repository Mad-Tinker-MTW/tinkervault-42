# ROADMAP
**TinkerVault 42**

---

## Stage 1: Audit Remediation — Target 2026-05-31

Fix all bugs and priority WARNs. Clean build. Vault round-trip verified.

Key items: BUG-01 date validation, BUG-02 full metadata capture/restore, BUG-03 delete original prompt, BUG-04 collision prompt, unique_path consolidation, npm dep pinning, mode switch path reset, Cargo.toml release profile.

Also: coord lookup helper button wired to madtinkersworkshop.com/coords.

**Gate:** cargo build --release passes. WRAP -> UNWRAP round-trip verified. Metadata preserved. Original deleted on confirm.

---

## Stage 2: Build, Sign, and Release Package — Target 2026-06-14

Complete Minisign signing. NSIS installer with verification as step one (fail = no install). Build madtinkersworkshop.com/coords lookup page. Produce full release package.

Release package contents:
```
TinkerVault 42 Setup.exe
release-manifest.json
release-manifest.sig
SHA256SUMS.txt
README.txt
VERIFY.txt
```

**Gate:** Signed installer produced. Verification step blocks install on failure. Coord lookup page live. Release package posted to GitHub and madtinkersworkshop.com.

---

## Stage 3: Test Coverage — Target 2026-07-31

Rust unit tests for all critical modules. Integration test for full wrap/unwrap round-trip including metadata preservation. cargo test gate.

**Gate:** cargo test passes. All critical paths covered including BUG-01 date edge cases and BUG-02 metadata round-trip.

---

## Stage 4: CTF Validation — Target 2026-09-01

Build challenge vaults using Beast Mode. Post to HackTheBox or similar. No flags, just the vault files and the challenge. Track attempts.

**Gate:** Challenge posted. 90-day window with no successful crack is meaningful validation.

---

## v1.0 Release — Target 2026-08-01

Signed installer, passing tests, complete documentation suite, coord lookup live.

---

## Future

**Mobile (Android):**
Tauri 2 supports Android. GPS makes coordinate input trivial on phone. Argon2id memory profile decision needed before viable (1 GB Beast Mode is not practical on most phones). Interim path: desktop-to-phone tunnel, wrap on desktop, carry vault on phone, unwrap via tunnel.

**Anonymity Tool (standalone):**
Strip or spoof file metadata. Separate MTW Security Division tool. Uses the same metadata schema established in TinkerVault 42 payload. Not a TinkerVault dependency.

**Known-place table expansion:**
Curated static table of landmark-quality places with permanent coordinates. Reduces SHA256 fallback usage for common memorable locations.

**Microsoft code signing:**
Add when TinkerVault is ready for broader public distribution.

**Streaming large file support:**
Removes RAM ceiling for files over ~500 MB. Deferred until needed.
