import { useState } from "react";
import SkillsTab from "./tabs/SkillsTab";
import GateTab from "./tabs/GateTab";
import "./App.css";

type Tab = "skills" | "gate";
type EditMap = Record<string, number>;

export default function App() {
  const [tab, setTab] = useState<Tab>("skills");
  const [edits, setEdits] = useState<EditMap>({});

  const changedCount = Object.keys(edits).length;

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
        </nav>
        <button className="export" disabled={changedCount === 0} title="Coming next">
          Export mod{changedCount ? ` (${changedCount})` : ""}
        </button>
      </header>

      <main className="content">
        {tab === "skills" ? (
          <SkillsTab edits={edits} setEdits={setEdits} />
        ) : (
          <GateTab />
        )}
      </main>
    </div>
  );
}
