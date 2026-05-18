# CLAUDE.md — TinkerVault 42

Guidance for Claude Code working in this repo. Read this fully before touching anything.

---

## Who You're Working For

Mad Tinker (Franky / Francisco De La Paz). MTW Security Division. U.S. Army vet, maker, problem-solver. Runs on KISS principles. No fluff, no over-engineering, no rewrites when a surgical edit will do. One-pass complete builds preferred. Plain language only, no em dashes, no formal polish.

When in doubt: do the job, report what you did, flag what needs a decision. Don't ask questions you can answer yourself by reading the code.

---

## Project

**TinkerVault 42** — Rust/Tauri desktop app. Deterministic multi-factor file and folder encryption. Windows-only. MTW internal tool, not a public product.

The core idea: five lock factors (things you know + where you were + when + sky) combine deterministically to derive an encryption key. No key files. No password managers. The key lives only in your memory and the universe.

---

## Commands

**Dev:**
```powershell
npm run dev:tauri          # Hot-reload (Vite + Tauri together)
npm run dev                # Frontend only, no Tauri backend
```

**Build:**
```powershell
npm run build:windows
# or equivalent:
powershell -ExecutionPolicy Bypass -File scripts/build_windows.ps1
```

**Outputs:**
```
src-tauri\target\release\tinkervault-42.exe
src-tauri\target\release\bundle\nsis\TinkerVault 42_42.1.0_x64-setup.exe
```

**Rust only (faster for backend work):**
```powershell
cd src-tauri
cargo check          # Fast syntax/type check
cargo build --release
cargo test
```

**Signing (optional):**
```powershell
powershell -ExecutionPolicy Bypass -File scripts/sign_manifest.ps1 -ReleaseFile "<path>"
```

---

## Architecture

### Stack

- Frontend: React 18 + Vite, Framer Motion, Lucide icons, Tauri JS API
- Backend: Rust, compiled into the Tauri binary (no Python sidecar)
- IPC: Two Tauri commands only: `wrap_payload`, `unwrap_vault`

### Data Flow

```
User Input (5 lock factors)
  seed phrase + place + date + sky time + star
        |
  seedforge.rs  -- PolyGlyph95-42 (calls geoastro.rs + nautical57.rs internally)
        |         produces material string
  payload.rs    -- file/folder serialize + SHA256 integrity  [built before key derivation]
        |
  crypto.rs     -- Argon2id Beast Mode: 1 GB RAM, 4 iterations, 1 thread -> 32-byte key
        |
  vault_format.rs -- AES-256-GCM encrypt/decrypt, TVLT42-1 binary format
```

Note: geoastro.rs and nautical57.rs are called inside seedforge.rs, not as separate pipeline
steps. payload serialization happens before key derivation to avoid holding 1 GB RAM during I/O.

### Modules (`src-tauri/src/`)

| File | Job |
|---|---|
| `lib.rs` | Tauri command handlers: `wrap_payload`, `unwrap_vault`, `clear_local_cache` |
| `vault_format.rs` | TVLT42-1 header serialization, encrypt/decrypt dispatch |
| `crypto.rs` | Argon2id KDF + AES-256-GCM |
| `seedforge.rs` | PolyGlyph95-42: 5 factors -> deterministic seed |
| `geoastro.rs` | Offline coordinate resolution, date/time normalization, alt/az math |
| `nautical57.rs` | Fixed 57-star catalog with RA/Dec |
| `payload.rs` | File and folder wrap/unwrap with SHA256 checksums |
| `errors.rs` | `VaultError` enum and `Result<T>` alias |

### Vault Format (TVLT42-1)

Binary header (unencrypted):
```
magic[8] | cipher_id[1] | argon2_params[12] | salt[32] | nonce[12]
```
Followed by AES-256-GCM ciphertext of a JSON payload (file metadata + base64 content).

### UI (`src/App.jsx`)

Two modes:
- WRAP: encrypts file or folder into a `.TinkerVault` file
- UNWRAP: decrypts a `.TinkerVault` file back to originals

Both modes collect all 5 lock factors and validate before invoking the Tauri command.

---

## Hard Constraints — Do Not Touch

- Argon2id Beast Mode: 1 GB RAM, 4 iterations, 1 thread. Non-negotiable. Do not reduce.
- Place resolution is offline and SHA256 deterministic. No geocoding APIs.
- All 5 lock factors must match exactly to reproduce the vault key. Order and casing matter in `seedforge.rs`.
- TVLT42-1 magic bytes and header layout are frozen. Changing them breaks existing vaults.

If a task would require touching any of the above, stop and flag it before proceeding.

---

## How to Work

- Surgical edits over full rewrites. Change only what needs changing.
- If something is broken, identify root cause first, then fix. Don't patch symptoms.
- `cargo check` before any full build to catch errors fast.
- Report what you did in plain language. Flag anything that needs a decision from Franky.
- Don't generate boilerplate comments. Code should speak for itself.

---

## Doc Standards (for reports and documentation)

When producing markdown reports or documentation for this project:

- Plain language, direct, no filler
- Headers for navigation, not decoration
- Tables for module/file references
- Code blocks for all commands and file paths
- No bullet soup -- use prose or tables instead
- Flag open issues clearly with a dedicated section
- Status labels: PASS, FAIL, WARN, TODO

---

## MTW Context

- Operator: Mad Tinker / Francisco De La Paz
- Division: MTW Security Division
- Parent org: 4Kings Enterprises, Altus OK
- Machine: TINKERSWORKSHOP (Ryzen 9 9900X, RTX 4070 Ti SUPER, 128GB DDR5)
- Drive layout: Workshop (C:), Vulcain's Forge (Q:), Library of Alexandria (V:), Ley Lines (X:)
- This project lives on Q: (Vulcain's Forge)
