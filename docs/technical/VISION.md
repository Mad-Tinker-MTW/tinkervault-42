# VISION
**TinkerVault 42**

---

## What It Is

TinkerVault 42 is a memoryless encryption tool. It does not store your key, your factors, or any metadata about what you encrypted. The key exists only in the geometry of a moment: where you were, when you were there, what was overhead, and what you were thinking.

If you can recall all five factors exactly, you can open any vault you ever created. If you cannot, the data is gone.

This is by design. Not a limitation. A feature.

---

## The Concept

Most encryption tools are as secure as their weakest link: the password manager, the key file on the same drive, the cloud backup, the browser autofill. TinkerVault 42 has none of those.

The key is a moment. You walked across the George Washington Bridge. You looked up at Sirius. You know what time it was in Zulu. You know what you were thinking. Nobody else was standing in that exact spot at that exact time with that exact thought.

The stellar position math is real. The app calculates exactly where that star was in the sky at your coordinates at that moment using standard spherical astronomy. Your memory and the universe agree on the answer. An attacker has to reconstruct your entire moment in time to even begin.

---

## Why the Seed Interleaving Matters

The seed phrase is not processed as a block. It is split into three segments and distributed into the lock chain between the geo and astro layers. An attacker must break two locks before seeing the first seed segment. Each subsequent segment is gated behind another lock.

There is no single attack surface. Every layer depends on the previous one being solved first. You cannot parallelize the attack.

---

## Design Philosophy

- Deterministic over random: same inputs always produce the same key, forever
- Offline over networked: no API, no telemetry, no external dependency at runtime
- Simple over flexible: two modes, five factors, one button
- Forensically complete: the unwrapped file is identical to the original, timestamps, attributes, owner, embedded metadata and all
- Internal over commercial: built for MTW Security Division, not mass distribution

---

## MTW Security Division Product Stack

TinkerVault 42 is the first tool in the MTW Security Division lineup:

**TinkerVault 42 (this tool)**
Wrap and unwrap files with full metadata preservation. The vault.

**Anonymity Tool (future, standalone)**
Strip or spoof file metadata: author, timestamps, owner, Zone.Identifier, NTFS alternate data streams, embedded document properties. Works on any file. Not a TinkerVault dependency, a separate tool that happens to share the metadata schema.

**AI Fingerprint Scrubber (exists)**
Strip AI identity markers from generated content. Separate tool, separate attack surface.

A user can run a file through the AI scrubber, then the anonymity tool, then TinkerVault and have a file with no AI markers, no metadata provenance, and strong deterministic encryption. That is the Security Division stack working together.

---

## Long-Term Direction

The vault format, key derivation pipeline, and star catalog are frozen at v1.0. Future versions may add convenience but will never change the core cryptographic behavior. A vault created with v1.0 must open with any future version.

**Mobile (future):** Tauri 2 supports Android. The GPS on a phone makes coordinate input trivial, one tap locks your exact position. The Argon2id memory requirement needs a mobile profile decision before this is viable. Desktop-to-phone tunnel workflow is the interim path.

**CTF validation:** Release challenge vaults on HackTheBox or similar. If nobody cracks them the security claim has real-world evidence behind it.

**Signing:** MTW release verification via Minisign. Manifest signed with MTW private key, public key embedded in installer. Verification is step one of the NSIS install sequence. Fails verification, does not install.

**Microsoft code signing:** Add when TinkerVault is ready for broader public distribution.

---

## What It Is Not

- Not a password manager
- Not a cloud storage solution
- Not recoverable without all five factors
- Not responsible for forgotten keys
- Not government certified or Microsoft verified (yet)
- Not claiming to be unbreakable, claiming to be MTW-built and MTW-signed
