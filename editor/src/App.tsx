import { useState } from "react";
import { openPath } from "@tauri-apps/plugin-opener";
import { open } from "@tauri-apps/plugin-dialog";
import SkillsTab from "./tabs/SkillsTab";
import GateTab from "./tabs/GateTab";
import PlayerTab from "./tabs/PlayerTab";
import SetupScreen from "./SetupScreen";
import { api } from "./api";
import "./App.css";

type Tab = "skills" | "gate" | "player";
type EditMap = Record<string, number>;
type DisabledMap = Record<string, boolean>;

export default function App() {
  const [tab, setTab] = useState<Tab>("skills");
  const [edits, setEdits] = useState<EditMap>({});
  const [disabled, setDisabled] = useState<DisabledMap>({});
  const [passiveEdits, setPassiveEdits] = useState<EditMap>({});
  const [playerEdits, setPlayerEdits] = useState<EditMap>({});
  const [modName, setModName] = useState("MyWolcenMod");
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [ready, setReady] = useState(false);

  async function doImport() {
    const dir = await open({
      directory: true,
      title: "Selecione a pasta do mod (a que contém 'Umbra')",
    });
    if (!dir || typeof dir !== "string") return;
    setImporting(true);
    setStatus("Importando mod…");
    try {
      const r = await api.importMod(dir);
      setEdits((prev) => {
        const n = { ...prev };
        for (const e of r.skill_edits) n[`${e.file}|${e.uid}|${e.element}|${e.attr}`] = e.value;
        return n;
      });
      setPassiveEdits((prev) => {
        const n = { ...prev };
        for (const e of r.passive_edits) n[`${e.file}|${e.node}|${e.eim}|${e.attr}`] = e.value;
        return n;
      });
      setPlayerEdits((prev) => {
        const n = { ...prev };
        for (const e of r.player_edits) n[`${e.file}|${e.element}|${e.attr}`] = e.value;
        return n;
      });
      const total = r.skill_edits.length + r.passive_edits.length + r.player_edits.length;
      setStatus(
        `✓ Importado: ${total} edição(ões) de ${r.files} arquivo(s).` +
          (r.skipped.length ? ` (${r.skipped.length} arquivo(s) ignorado(s))` : "")
      );
    } catch (e) {
      setStatus(`✗ erro ao importar: ${e}`);
    } finally {
      setImporting(false);
    }
  }

  const changedCount =
    Object.keys(edits).length +
    Object.values(disabled).filter(Boolean).length +
    Object.keys(passiveEdits).length +
    Object.keys(playerEdits).length;

  async function doExport() {
    setExporting(true);
    setStatus(null);
    try {
      // skill edits (disabled modifiers override with value 0)
      const skillMap = new Map<string, number>();
      for (const [k, v] of Object.entries(edits)) skillMap.set(k, v);
      for (const [k, off] of Object.entries(disabled)) if (off) skillMap.set(k, 0);
      const skill_edits = Array.from(skillMap, ([k, value]) => {
        const [file, uid, element, attr] = k.split("|");
        return { file, uid, element, attr, value };
      });
      const passive_edits = Object.entries(passiveEdits).map(([k, value]) => {
        const [file, node, eim, attr] = k.split("|");
        return { file, node, eim, attr, value };
      });
      const player_edits = Object.entries(playerEdits).map(([k, value]) => {
        const [file, element, attr] = k.split("|");
        return { file, element, attr, value };
      });

      const res = await api.exportMod({
        mod_name: modName || "MyWolcenMod",
        skill_edits,
        passive_edits,
        player_edits,
      });
      setStatus(`✓ ${res.changes} edição(ões) em ${res.files} arquivo(s) → ${res.folder}`);
      try {
        await openPath(res.folder);
      } catch {
        /* opening the folder is best-effort */
      }
    } catch (e) {
      setStatus(`✗ erro: ${e}`);
    } finally {
      setExporting(false);
    }
  }

  if (!ready) {
    return <SetupScreen onReady={() => setReady(true)} />;
  }

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">Wolcen Mod Editor</div>
        <nav className="tabs">
          <button className={tab === "skills" ? "active" : ""} onClick={() => setTab("skills")}>
            Skills
          </button>
          <button className={tab === "gate" ? "active" : ""} onClick={() => setTab("gate")}>
            Gate of Fates
          </button>
          <button className={tab === "player" ? "active" : ""} onClick={() => setTab("player")}>
            Player
          </button>
        </nav>
        <div className="export-bar">
          <button className="import-btn" disabled={importing || exporting} onClick={doImport}>
            {importing ? "Importando…" : "Importar mod"}
          </button>
          <input
            className="mod-name"
            value={modName}
            onChange={(e) => setModName(e.target.value)}
            placeholder="nome do mod"
            title="Nome do mod"
          />
          <button className="export" disabled={changedCount === 0 || exporting} onClick={doExport}>
            {exporting ? "Exportando…" : `Export mod${changedCount ? ` (${changedCount})` : ""}`}
          </button>
        </div>
      </header>

      {status && <div className="statusbar">{status}</div>}

      <main className="content">
        {tab === "skills" && (
          <SkillsTab
            edits={edits}
            setEdits={setEdits}
            disabled={disabled}
            setDisabled={setDisabled}
          />
        )}
        {tab === "gate" && <GateTab edits={passiveEdits} setEdits={setPassiveEdits} />}
        {tab === "player" && <PlayerTab edits={playerEdits} setEdits={setPlayerEdits} />}
      </main>
    </div>
  );
}
