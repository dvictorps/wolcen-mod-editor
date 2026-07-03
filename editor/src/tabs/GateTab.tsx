import { useEffect, useMemo, useRef, useState } from "react";
import { api, PassiveSection, PassiveNode } from "../api";

const VB = 2100;
const CX = VB / 2;
const CY = VB / 2;
const INNER_R = 320;
const OUTER_R = 1000;

type Placed = {
  section: string;
  pstFile: string;
  node: PassiveNode;
  x: number;
  y: number;
  search: string;
};
type EditMap = Record<string, number>;

function nodeRadius(rarity: number) {
  return rarity >= 3 ? 8 : rarity === 2 ? 6 : 4;
}
const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
const fieldKey = (file: string, node: string, eim: string, attr: string) =>
  `${file}|${node}|${eim}|${attr}`;

export default function GateTab({
  edits,
  setEdits,
}: {
  edits: EditMap;
  setEdits: (fn: (prev: EditMap) => EditMap) => void;
}) {
  const [sections, setSections] = useState<PassiveSection[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<Placed | null>(null);
  const [hovered, setHovered] = useState<Placed | null>(null);
  const [query, setQuery] = useState("");

  const [scale, setScale] = useState(1);
  const [tx, setTx] = useState(0);
  const [ty, setTy] = useState(0);
  const svgRef = useRef<SVGSVGElement | null>(null);
  const drag = useRef<{ x: number; y: number; moved: boolean } | null>(null);

  useEffect(() => {
    api
      .listSections()
      .then(async (list) => {
        const loaded = await Promise.all(list.map((s) => api.getSection(s.name)));
        setSections(loaded);
      })
      .catch((e) => setError(String(e)));
  }, []);

  const { placed, edges, labels } = useMemo(() => {
    const placed: Placed[] = [];
    const edges: { x1: number; y1: number; x2: number; y2: number }[] = [];
    const labels: { x: number; y: number; text: string; angle: number }[] = [];
    const n = sections.length || 1;
    const step = (2 * Math.PI) / n;
    const span = step * 0.98;

    sections.forEach((sec, i) => {
      const base = i * step - Math.PI / 2;
      const byName = new Map<string, { x: number; y: number }>();
      for (const node of sec.nodes) {
        const theta = base + (node.angle - 0.5) * span;
        const r = INNER_R + Math.pow(node.pos, 0.8) * (OUTER_R - INNER_R);
        const x = CX + r * Math.cos(theta);
        const y = CY + r * Math.sin(theta);
        byName.set(node.name, { x, y });
        // searchable text: node names + what each effect does
        const eff = node.effects
          .map((e) => e.label + " " + e.fields.map((f) => f.attr).join(" "))
          .join(" ");
        const search = (node.display_name + " " + node.name + " " + eff).toLowerCase();
        placed.push({ section: sec.name, pstFile: sec.pst_file, node, x, y, search });
      }
      for (const node of sec.nodes) {
        const from = byName.get(node.name);
        if (!from) continue;
        for (const t of node.unlock) {
          const to = byName.get(t);
          if (to) edges.push({ x1: from.x, y1: from.y, x2: to.x, y2: to.y });
        }
      }
      const lr = OUTER_R + 30;
      let a = (base * 180) / Math.PI;
      if (a > 90) a -= 180;
      if (a < -90) a += 180;
      labels.push({ x: CX + lr * Math.cos(base), y: CY + lr * Math.sin(base), text: sec.name, angle: a });
    });
    return { placed, edges, labels };
  }, [sections]);

  const matches = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return null as Set<Placed> | null;
    const set = new Set<Placed>();
    for (const p of placed) if (p.search.includes(q)) set.add(p);
    return set;
  }, [query, placed]);

  useEffect(() => {
    if (!matches || matches.size === 0) return;
    const first = matches.values().next().value as Placed;
    const s = 2.6;
    setScale(s);
    setTx(VB / 2 - s * first.x);
    setTy(VB / 2 - s * first.y);
  }, [matches]);

  function toSvg(clientX: number, clientY: number) {
    const rect = svgRef.current!.getBoundingClientRect();
    return { x: ((clientX - rect.left) / rect.width) * VB, y: ((clientY - rect.top) / rect.height) * VB };
  }
  function onWheel(e: React.WheelEvent) {
    e.preventDefault();
    const { x: mx, y: my } = toSvg(e.clientX, e.clientY);
    const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
    const ns = clamp(scale * factor, 0.6, 10);
    setTx(mx - ((mx - tx) / scale) * ns);
    setTy(my - ((my - ty) / scale) * ns);
    setScale(ns);
  }
  function onDown(e: React.MouseEvent) {
    drag.current = { x: e.clientX, y: e.clientY, moved: false };
  }
  function onMove(e: React.MouseEvent) {
    if (!drag.current) return;
    const dx = e.clientX - drag.current.x;
    const dy = e.clientY - drag.current.y;
    if (Math.abs(dx) + Math.abs(dy) > 3) drag.current.moved = true;
    const rect = svgRef.current!.getBoundingClientRect();
    setTx((t) => t + (dx / rect.width) * VB);
    setTy((t) => t + (dy / rect.height) * VB);
    drag.current.x = e.clientX;
    drag.current.y = e.clientY;
  }
  function onUp() {
    drag.current = null;
  }
  function resetView() {
    setScale(1);
    setTx(0);
    setTy(0);
  }

  return (
    <div className="split">
      <section className="wheel-wrap">
        {error && <div className="error">{error}</div>}
        <div className="wheel-toolbar">
          <input
            className="node-search"
            placeholder="Buscar por nome ou efeito… (ex: armor, attack speed)"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          {matches && <span className="muted small">{matches.size} nó(s)</span>}
          <button onClick={resetView}>Reset view</button>
          <span className="muted small">scroll = zoom · arraste = mover</span>
          {hovered && <span className="hover-name">{hovered.node.display_name}</span>}
        </div>
        <svg
          ref={svgRef}
          viewBox={`0 0 ${VB} ${VB}`}
          className={"wheel" + (drag.current ? " grabbing" : "")}
          onWheel={onWheel}
          onMouseDown={onDown}
          onMouseMove={onMove}
          onMouseUp={onUp}
          onMouseLeave={onUp}
        >
          <g transform={`translate(${tx},${ty}) scale(${scale})`}>
            <circle cx={CX} cy={CY} r={INNER_R - 12} className="hub" />
            {edges.map((e, i) => (
              <line key={i} x1={e.x1} y1={e.y1} x2={e.x2} y2={e.y2} className="edge" />
            ))}
            {placed.map((p, i) => {
              const isSel = selected?.node.name === p.node.name && selected?.section === p.section;
              const isMatch = matches?.has(p) ?? false;
              const dim = matches && !isMatch;
              return (
                <circle
                  key={i}
                  cx={p.x}
                  cy={p.y}
                  r={nodeRadius(p.node.rarity) + (isSel || isMatch ? 3 : 0)}
                  className={
                    "node r" + p.node.rarity +
                    (isSel ? " sel" : "") +
                    (isMatch ? " match" : "") +
                    (dim ? " dim" : "")
                  }
                  onMouseEnter={() => setHovered(p)}
                  onClick={() => {
                    if (!drag.current?.moved) setSelected(p);
                  }}
                />
              );
            })}
            {labels.map((l, i) => (
              <text key={i} x={l.x} y={l.y} className="wheel-label" transform={`rotate(${l.angle} ${l.x} ${l.y})`}>
                {l.text}
              </text>
            ))}
          </g>
        </svg>
      </section>

      <aside className="node-panel">
        <div className="rail-title">Node</div>
        {!selected && <div className="muted small">Clique num nó da roda (ou busque acima).</div>}
        {selected && (
          <div>
            <div className="node-title">{selected.node.display_name}</div>
            <div className="kv"><span>id</span><b>{selected.node.name}</b></div>
            <div className="kv"><span>section</span><b>{selected.section}</b></div>
            {selected.node.effects.length === 0 && (
              <div className="muted small" style={{ marginTop: 10 }}>Sem efeitos numéricos editáveis.</div>
            )}
            {selected.node.effects.map((eff, ei) => (
              <div className="effect" key={ei}>
                <div className="effect-label">{eff.label}</div>
                {eff.fields.map((f) => {
                  const key = fieldKey(selected.pstFile, selected.node.name, eff.eim, f.attr);
                  const val = key in edits ? edits[key] : f.value;
                  const changed = key in edits && edits[key] !== f.value;
                  return (
                    <label className="field" key={f.attr}>
                      <span className="field-name" title={eff.eim}>{f.attr}</span>
                      <input
                        type="number"
                        step="any"
                        value={val}
                        className={changed ? "changed" : ""}
                        onChange={(e) => {
                          const nv = parseFloat(e.target.value);
                          setEdits((prev) => ({ ...prev, [key]: Number.isNaN(nv) ? f.value : nv }));
                        }}
                      />
                      {changed && <span className="orig">was {f.value}</span>}
                    </label>
                  );
                })}
              </div>
            ))}
          </div>
        )}
      </aside>
    </div>
  );
}
