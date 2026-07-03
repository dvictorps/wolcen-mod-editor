import { useEffect, useMemo, useState } from "react";
import { api, PassiveSection, PassiveNode } from "../api";

const CX = 450;
const CY = 450;
const INNER_R = 95;
const OUTER_R = 410;

type Placed = {
  section: string;
  node: PassiveNode;
  x: number;
  y: number;
};

function nodeRadius(rarity: number) {
  return rarity >= 3 ? 11 : rarity === 2 ? 7.5 : 4.5;
}

export default function GateTab() {
  const [sections, setSections] = useState<PassiveSection[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<Placed | null>(null);

  useEffect(() => {
    api
      .listSections()
      .then(async (list) => {
        const loaded = await Promise.all(list.map((s) => api.getSection(s.name)));
        setSections(loaded);
      })
      .catch((e) => setError(String(e)));
  }, []);

  // Compute node screen positions + intra-section edges.
  const { placed, edges, labels } = useMemo(() => {
    const placed: Placed[] = [];
    const edges: { x1: number; y1: number; x2: number; y2: number }[] = [];
    const labels: { x: number; y: number; text: string; angle: number }[] = [];
    const n = sections.length || 1;
    const step = (2 * Math.PI) / n;
    const span = step * 0.82;

    sections.forEach((sec, i) => {
      const base = i * step - Math.PI / 2;
      const byName = new Map<string, { x: number; y: number }>();
      const local: Placed[] = [];

      for (const node of sec.nodes) {
        const theta = base + (node.angle - 0.5) * span;
        const r = INNER_R + node.pos * (OUTER_R - INNER_R);
        const x = CX + r * Math.cos(theta);
        const y = CY + r * Math.sin(theta);
        byName.set(node.name, { x, y });
        const p: Placed = { section: sec.name, node, x, y };
        local.push(p);
        placed.push(p);
      }
      // edges within the section
      for (const node of sec.nodes) {
        const from = byName.get(node.name);
        if (!from) continue;
        for (const target of node.unlock) {
          const to = byName.get(target);
          if (to) edges.push({ x1: from.x, y1: from.y, x2: to.x, y2: to.y });
        }
      }
      // section label just past the outer edge
      const lr = OUTER_R + 22;
      labels.push({
        x: CX + lr * Math.cos(base),
        y: CY + lr * Math.sin(base),
        text: sec.name,
        angle: (base * 180) / Math.PI,
      });
    });
    return { placed, edges, labels };
  }, [sections]);

  return (
    <div className="split">
      <section className="wheel-wrap">
        {error && <div className="error">{error}</div>}
        {sections.length === 0 && !error && <div className="muted">loading tree…</div>}
        <svg viewBox="0 0 900 900" className="wheel">
          <circle cx={CX} cy={CY} r={INNER_R - 8} className="hub" />
          {edges.map((e, i) => (
            <line key={i} x1={e.x1} y1={e.y1} x2={e.x2} y2={e.y2} className="edge" />
          ))}
          {placed.map((p, i) => {
            const isSel = selected?.node.name === p.node.name && selected?.section === p.section;
            return (
              <circle
                key={i}
                cx={p.x}
                cy={p.y}
                r={nodeRadius(p.node.rarity)}
                className={
                  "node r" + p.node.rarity + (isSel ? " sel" : "")
                }
                onClick={() => setSelected(p)}
              >
                <title>{`${p.section} · ${p.node.name} (rarity ${p.node.rarity})`}</title>
              </circle>
            );
          })}
          {labels.map((l, i) => (
            <text
              key={i}
              x={l.x}
              y={l.y}
              className="wheel-label"
              transform={
                Math.abs(l.angle) > 90
                  ? `rotate(${l.angle + 180} ${l.x} ${l.y})`
                  : `rotate(${l.angle} ${l.x} ${l.y})`
              }
            >
              {l.text}
            </text>
          ))}
        </svg>
      </section>

      <aside className="node-panel">
        <div className="rail-title">Node</div>
        {!selected && <div className="muted small">Click a node on the wheel.</div>}
        {selected && (
          <div>
            <div className="node-title">{selected.node.name}</div>
            <div className="kv"><span>section</span><b>{selected.section}</b></div>
            <div className="kv"><span>rarity</span><b>{selected.node.rarity}</b></div>
            <div className="kv"><span>angle</span><b>{selected.node.angle}</b></div>
            <div className="kv"><span>pos</span><b>{selected.node.pos}</b></div>
            <div className="kv"><span>unlocks</span><b>{selected.node.unlock.join(", ") || "—"}</b></div>
            <div className="muted small" style={{ marginTop: 12 }}>
              Editing node effects (from PassiveEffects.xml) comes next — this proves the
              wheel renders from real data and nodes are selectable.
            </div>
          </div>
        )}
      </aside>
    </div>
  );
}
