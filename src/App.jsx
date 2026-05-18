import React, { useEffect, useMemo, useState } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { open, message, ask } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  Lock,
  Unlock,
  Folder,
  File,
  Shield,
  Star,
  MapPin,
  Calendar,
  Clock3,
  Sparkles,
  Eye,
  EyeOff,
  Flame,
  HelpCircle,
  Trash2,
} from "lucide-react";

const stars = [
  "Acamar", "Achernar", "Acrux", "Adhara", "Aldebaran", "Alioth", "Alkaid",
  "Al Na'ir", "Alnilam", "Alphard", "Alphecca", "Alpheratz", "Altair",
  "Ankaa", "Antares", "Arcturus", "Atria", "Avior", "Bellatrix", "Betelgeuse",
  "Canopus", "Capella", "Deneb", "Denebola", "Diphda", "Dubhe", "Elnath",
  "Eltanin", "Enif", "Fomalhaut", "Gacrux", "Gienah", "Hadar", "Hamal",
  "Kaus Australis", "Kochab", "Markab", "Menkar", "Menkent", "Miaplacidus",
  "Mirfak", "Nunki", "Peacock", "Polaris", "Pollux", "Procyon", "Rasalhague",
  "Regulus", "Rigel", "Rigil Kentaurus", "Sabik", "Schedar", "Shaula",
  "Sirius", "Spica", "Suhail", "Vega", "Zubenelgenubi"
];

function computeExpectedVaultPath(inputPath, payloadType) {
  if (!inputPath) return null;
  const useBackslash = inputPath.includes("\\");
  const sep = useBackslash ? "\\" : "/";
  const parts = inputPath.split(/[/\\]/);
  const filename = parts[parts.length - 1];
  const parent = parts.slice(0, -1).join(sep);
  const dotIdx = filename.lastIndexOf(".");
  const stem =
    payloadType === "folder" || dotIdx <= 0
      ? filename
      : filename.substring(0, dotIdx);
  return (parent ? parent + sep : "") + stem + ".TinkerVault";
}

const glassAnim = {
  initial: { opacity: 0, y: 14, scale: 0.99 },
  animate: { opacity: 1, y: 0, scale: 1 },
  transition: { duration: 0.35, ease: [0.22, 1, 0.36, 1] },
};

function Field({ icon: Icon, label, value, setValue, type = "text", placeholder }) {
  return (
    <label className="field">
      <span className="field-label">
        {Icon && <Icon size={14} />}
        {label}
      </span>
      <input
        type={type}
        value={value}
        placeholder={placeholder}
        onChange={(e) => setValue(e.target.value)}
      />
    </label>
  );
}

export default function App() {
  const [mode, setMode] = useState("wrap");
  const [payloadPath, setPayloadPath] = useState("");
  const [payloadType, setPayloadType] = useState("file");
  const [seed, setSeed] = useState("");
  const [showSeed, setShowSeed] = useState(false);
  const [place, setPlace] = useState("40.814870,-73.888250");
  const [date, setDate] = useState("1985-09-20");
  const [skyTime, setSkyTime] = useState("21:00");
  const [star, setStar] = useState("Sirius");
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);
  const [lastOutput, setLastOutput] = useState("");

  useEffect(() => {
    let unlisten;
    getCurrentWebview().onDragDropEvent((event) => {
      const payload = event.payload;
      if (payload?.type === "drop" && payload.paths?.length) {
        const path = payload.paths[0];
        setPayloadPath(path);
        const lower = path.toLowerCase();
        setPayloadType(lower.endsWith(".tinkervault") ? "vault" : "file/folder");
        setStatus("Payload dropped");
      }
    }).then((fn) => { unlisten = fn; }).catch(() => {});
    return () => { if (unlisten) unlisten(); };
  }, []);

  const selectedText = useMemo(() => {
    if (!payloadPath) return "Drop or choose a file/folder";
    return payloadPath;
  }, [payloadPath]);

  async function pickPayload(kind) {
    try {
      let selected;
      if (mode === "unwrap") {
        selected = await open({
          multiple: false,
          directory: false,
          filters: [{ name: "TinkerVault", extensions: ["TinkerVault"] }],
        });
        setPayloadType("vault");
      } else if (kind === "folder") {
        selected = await open({ multiple: false, directory: true });
        setPayloadType("folder");
      } else {
        selected = await open({ multiple: false, directory: false });
        setPayloadType("file");
      }

      if (typeof selected === "string") {
        setPayloadPath(selected);
        setStatus("Payload selected");
        setLastOutput("");
      }
    } catch (err) {
      setStatus(`File picker error: ${err}`);
      await message(String(err), { title: "TinkerVault picker error", kind: "error" });
    }
  }

  function validateInputs() {
    if (!payloadPath) throw new Error("Choose a payload or vault first.");
    if (!seed.trim()) throw new Error("Enter your seed phrase.");
    if (!place.trim()) throw new Error("Enter a place or coordinates.");
    if (!date.trim()) throw new Error("Enter a date.");
    if (!skyTime.trim()) throw new Error("Enter a sky time.");
    if (!star.trim()) throw new Error("Choose a star.");
  }

  async function runAction() {
    let overwrite = false;

    try {
      validateInputs();

      // WARN-08: guard against default values on wrap
      if (mode === "wrap") {
        const DEFAULTS = { place: "40.814870,-73.888250", date: "1985-09-20", skyTime: "21:00" };
        const flagged = [];
        if (place.trim() === DEFAULTS.place) flagged.push("Place");
        if (date.trim() === DEFAULTS.date) flagged.push("Date");
        if (skyTime.trim() === DEFAULTS.skyTime) flagged.push("Sky Time");
        if (flagged.length > 0) {
          const proceed = await ask(
            `${flagged.join(" and ")} ${flagged.length === 1 ? "is" : "are"} still at default values. These are not secret.\n\nContinue wrapping?`,
            { title: "Default values detected", kind: "warning" }
          );
          if (!proceed) return;
        }
      }

      // BUG-04: pre-check collision for wrap before Beast Mode
      if (mode === "wrap") {
        const collision = await invoke("check_wrap_collision", { inputPath: payloadPath });
        if (collision) {
          const doOverwrite = await ask(
            `Output already exists:\n${collision}\n\nOverwrite?`,
            { title: "File collision", kind: "warning" }
          );
          if (!doOverwrite) return;
          overwrite = true;
        }
      }

      setBusy(true);
      setLastOutput("");
      setStatus("Working with Argon2id Beast Mode...");

      const request = { payloadPath, seed, place, date, skyTime, star, overwrite };

      if (mode === "wrap") {
        const result = await invoke("wrap_payload", { request });
        setLastOutput(result);

        // BUG-03: prompt to delete original after successful wrap
        const shouldDelete = await ask(
          `Wrapped successfully.\n\nDelete original? This cannot be undone.\n\n${payloadPath}`,
          { title: "Delete original?", kind: "warning" }
        );
        if (shouldDelete) {
          try {
            const delStatus = await invoke("delete_original", { path: payloadPath });
            setStatus(delStatus);
          } catch (delErr) {
            setStatus("Wrap OK. Delete failed.");
            await message(String(delErr), { title: "Delete error", kind: "error" });
          }
        } else {
          setStatus("Wrapped. Original kept.");
        }
      } else {
        // BUG-04: handle COLLISION error from unwrap after Beast Mode
        let result;
        try {
          result = await invoke("unwrap_vault", { request });
        } catch (unwrapErr) {
          const errStr = String(unwrapErr);
          if (errStr.startsWith("COLLISION:")) {
            const collidingPath = errStr.slice("COLLISION:".length);
            const doOverwrite = await ask(
              `Output already exists:\n${collidingPath}\n\nOverwrite? Beast Mode will run again.`,
              { title: "File collision", kind: "warning" }
            );
            if (!doOverwrite) return;
            result = await invoke("unwrap_vault", { request: { ...request, overwrite: true } });
          } else {
            throw unwrapErr;
          }
        }
        setLastOutput(result);
        setStatus("Unwrapped successfully");
        await message(result, { title: "TinkerVault", kind: "info" });
      }
    } catch (err) {
      setStatus(`Error: ${err}`);
      setLastOutput("");
      await message(String(err), { title: "TinkerVault error", kind: "error" });
    } finally {
      setBusy(false);
    }
  }

  async function showHelp() {
    await message(
`TinkerVault 42 quick use:

WRAP
1. Choose File, Choose Folder, or drag a payload into the drop zone.
2. Enter the five remembered lock parts:
   - Seed phrase
   - Place or coordinates
   - Date
   - Sky Time
   - Star
3. Click Wrap Payload.
4. A .TinkerVault file is created beside the original.

UNWRAP
1. Switch to Unwrap.
2. Choose or drag the .TinkerVault file.
3. Enter the exact same five lock parts.
4. Click Unwrap Vault.

IMPORTANT
- This final-core build is Rust-native. No Python sidecar.
- For the most repeatable final-core lock, use exact coordinates as the place input.
- Example: 40.814870,-73.888250
- The vault does not store your seed/place/date/time/star in readable form.
- Beast Mode may take a moment.`,
      { title: "TinkerVault Help", kind: "info" }
    );
  }

  async function clearCache() {
    try {
      const result = await invoke("clear_local_cache");
      setStatus(result);
      await message(result, { title: "TinkerVault cleanup", kind: "info" });
    } catch (err) {
      setStatus(`Cleanup error: ${err}`);
      await message(String(err), { title: "TinkerVault cleanup error", kind: "error" });
    }
  }

  return (
    <main className="app-shell">
      <div className="noise" />
      <div className="orb orb-red" />
      <div className="orb orb-blue" />

      <aside className="side-rail">
        <div className="brand">
          <div className="brand-mark"><Shield size={20} /></div>
          <div>
            <h1>TinkerVault</h1>
            <p>42</p>
          </div>
        </div>

        <div className="division">MTW Security Division</div>

        <section className="glass mini">
          <p className="section-kicker">Mode</p>
          <button className={mode === "wrap" ? "mode active" : "mode"} onClick={() => { setMode("wrap"); setPayloadPath(""); setPayloadType("file"); }} disabled={busy}>
            <Lock size={16} /> Wrap
          </button>
          <button className={mode === "unwrap" ? "mode active" : "mode"} onClick={() => { setMode("unwrap"); setPayloadPath(""); setPayloadType("vault"); }} disabled={busy}>
            <Unlock size={16} /> Unwrap
          </button>
        </section>

        <section className="tech-stack">
          <p><span>Vault</span> TVLT42-1</p>
          <p><span>KDF</span> Argon2id Beast</p>
          <p><span>Cipher</span> AES-256-GCM</p>
          <p><span>Stars</span> Nautical-57</p>
        </section>

        <div className="rail-actions">
          <button className="rail-btn" onClick={showHelp}><HelpCircle size={15} /> Help</button>
          <button className="rail-btn" onClick={clearCache}><Trash2 size={15} /> Clear cache</button>
        </div>
      </aside>

      <section className="workspace">
        <motion.header {...glassAnim} className="hero compact">
          <div>
            <p className="eyebrow"><Sparkles size={15} /> MTW Security Division</p>
            <h2>Private file and folder wrapping</h2>
            <p className="hero-copy">
              Rust-native final core. Enter the same five remembered lock parts to restore.
            </p>
          </div>
        </motion.header>

        <div className="content-grid">
          <motion.section {...glassAnim} className="glass payload-card">
            <div className="card-heading">
              <div>
                <p className="section-kicker">Step 1</p>
                <h3>{mode === "wrap" ? "Select payload" : "Select vault"}</h3>
              </div>
              <div className="pill">{payloadType}</div>
            </div>

            <div className="drop-zone">
              <div className="drop-icon">{payloadType === "folder" ? <Folder size={24} /> : <File size={24} />}</div>
              <div>
                <strong>{selectedText}</strong>
                <span>{mode === "wrap" ? "Drop or choose a file/folder." : "Drop or choose a .TinkerVault file."}</span>
              </div>
            </div>

            <div className="button-row">
              <button className="ghost-btn" onClick={() => pickPayload("file")} disabled={busy}><File size={16} /> File</button>
              <button className="ghost-btn" onClick={() => pickPayload("folder")} disabled={busy || mode === "unwrap"}><Folder size={16} /> Folder</button>
            </div>
          </motion.section>

          <motion.section {...glassAnim} className="glass lock-card">
            <div className="card-heading">
              <div>
                <p className="section-kicker">Step 2</p>
                <h3>Remembered lock</h3>
              </div>
              <button className="icon-btn" onClick={() => setShowSeed(!showSeed)} disabled={busy}>
                {showSeed ? <EyeOff size={17} /> : <Eye size={17} />}
              </button>
            </div>

            <div className="grid">
              <Field icon={Flame} label="Seed phrase" value={seed} setValue={setSeed} type={showSeed ? "text" : "password"} placeholder="Your private phrase" />
              <label className="field">
                <span className="field-label"><MapPin size={14} /> Place or coordinates</span>
                <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
                  <input
                    style={{ flex: 1 }}
                    type="text"
                    value={place}
                    onChange={(e) => setPlace(e.target.value)}
                  />
                  <button
                    className="ghost-btn"
                    style={{ flexShrink: 0, fontSize: "12px", padding: "4px 10px" }}
                    onClick={(e) => { e.preventDefault(); invoke("open_coords_url", { query: place }); }}
                    disabled={busy}
                  >
                    Look up
                  </button>
                </div>
              </label>
              <Field icon={Calendar} label="Date" value={date} setValue={setDate} placeholder="YYYY-MM-DD" />
              <Field icon={Clock3} label="Sky Time" value={skyTime} setValue={setSkyTime} placeholder="HH:MM" />

              <label className="field">
                <span className="field-label"><Star size={14} /> Nautical star</span>
                <select value={star} onChange={(e) => setStar(e.target.value)} disabled={busy}>
                  {stars.map((s) => <option key={s} value={s}>{s}</option>)}
                </select>
              </label>
            </div>
          </motion.section>
        </div>

        <motion.section {...glassAnim} className="glass action-card compact-action">
          <button className="primary-action" onClick={runAction} disabled={busy}>
            {mode === "wrap" ? <Lock size={19} /> : <Unlock size={19} />}
            {busy ? "Working..." : (mode === "wrap" ? "Wrap Payload" : "Unwrap Vault")}
          </button>
          <p>{lastOutput || status || "Beast Mode may take a moment during wrap or unwrap."}</p>
        </motion.section>
      </section>
    </main>
  );
}
