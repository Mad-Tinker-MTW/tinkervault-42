# TinkerVault 42 Final Core

MTW Security Division private file and folder wrapper.

## What this build changes

This is the Rust-native final-core move.

Removed:

```text
Python sidecar
PyInstaller
backend EXE
stdin backend bridge
command-line secret handoff
```

Current architecture:

```text
Tauri/React UI
→ Rust vault engine
```

## Final-core components

```text
Vault format: TVLT42-1
Geo lock: GeoAstroLock 42 Nautical-57
Glyph lock: PolyGlyph95-42
KDF: Argon2id Beast Mode
Cipher: AES-256-GCM
```

## Nautical-57

The app uses a fixed set of 57 nautical navigational stars.

User sees common/sailor names:

```text
Sirius
Vega
Polaris
Canopus
Arcturus
```

Engine uses internal IDs:

```text
NAV57_SIRIUS
NAV57_VEGA
NAV57_POLARIS
```

## Place rule

For best final-core repeatability, use exact coordinates:

```text
40.814870,-73.888250
```

If a plain place label is used, the Rust core derives a deterministic offline place vector from the normalized text. This keeps the engine offline and repeatable, but exact coordinates are preferred.

## Build

```powershell
npm install
npm run tauri build
```

Or:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_windows.ps1
```

## Outputs

```text
src-tauri\target\release\tinkervault-42.exe
src-tauri\target\release\bundle
```

## Not included yet

MTW signed release verification badge is planned but not wired in this package. A manifest helper script is included:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/sign_manifest.ps1 -ReleaseFile ".\src-tauri\target\release\bundle\nsis\TinkerVault 42_42.1.0_x64-setup.exe"
```

A real signing key/tool decision is still required.
