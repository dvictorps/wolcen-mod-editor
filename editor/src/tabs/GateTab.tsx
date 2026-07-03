import { useEffect, useMemo, useRef, useState } from "react";
import { api, PassiveSection, PassiveNode } from "../api";

const VB = 1800; // viewBox size — large so nodes spread along long spokes
const CX = VB / 2;
const CY = VB / 2;
const INNER_R = 150;
const OUTER_R = 890;

type Placed = { section: string; node: PassiveNode; x: number; y: number };

function nodeRadius(rarity: number) {
  return rarity >= 3 ? 11 : rarity === 2 ? 7 : 4.5;
}
const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));

export default function GateTab() {
  const [sections, setSections] = useState<PassiveSection[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<Placed | null>(null);
  const [hovered, setHovered] = useState<Placed | null>(null);

  // zoom / pan
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
        const r = INNER_R + node.pos * (OUTER_R - INNER_R);
        const x = CX + r * Math.cos(theta);
        const y = CY + r * Math.sin(theta);
        byName.set(node.name, { x, y });
        placed.push({ section: sec.name, node, x, y });
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

  function toSvg(clientX: number, clientY: number) {
    const rect = svgRef.current!.getBoundingClientRect();
    return {
      x: ((clientX - rect.left) / rect.width) * VB,
      y: ((clientY - rect.top) / rect.height) * VB,
    };
  }

  function onWheel(e: React.WheelEvent) {
    e.preventDefault();
    const { x: mx, y: my } = toSvg(e.clientX, e.clientY);
    const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
    const ns = clamp(scale * factor, 0.6, 10);
    const wx = (mx - tx) / scale;
    const wy = (my - ty) / scale;
    setTx(mx - wx * ns);
    setTy(my - wy * ns);
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
        {sections.length === 0 && !error && <div className="muted">loading tree…</div>}
        <div className="wheel-toolbar">
          <span className="muted small">scroll = zoom · arraste = mover</span>
          <button onClick={resetView}>Reset view</button>
          {hovered && <span className="hover-name">{hovered.section} · {hovered.node.name}</span>}
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
            <circle cx={CX} cy={CY} r={INNER_R - 10} className="hub" />
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
                  r={nodeRadius(p.node.rarity) + (isSel ? 3 : 0)}
                  className={"node r" + p.node.rarity + (isSel ? " sel" : "")}
                  onMouseEnter={() => setHovered(p)}
                  onClick={() => {
                    if (!drag.current?.moved) setSelected(p);
                  }}
                />
              );
            })}
            {labels.map((l, i) => (
              <text
                key={i}
                x={l.x}
                y={l.y}
                className="wheel-label"
                transform={`rotate(${l.angle} ${l.x} ${l.y})`}
              >
                {l.text}
              </text>
            ))}
          </g>
        </svg>
      </section>

      <aside className="node-panel">
        <div className="rail-title">Node</div>
        {!selected && <div className="muted small">Clique num nó da roda.</div>}
        {selected && (
          <div>
            <div className="node-title">{selected.node.name}</div>
            <div className="kv"><span>section</span><b>{selected.section}</b></div>
            <div className="kv"><span>rarity</span><b>{selected.node.rarity}</b></div>
            <div className="kv"><span>connections</span><b>{selected.node.unlock.length}</b></div>
            <div className="node-note">
              Editar o efeito deste nó é o próximo passo — falta mapear nó→stat
              (o <code>PassiveEffects.xml</code> usa ids próprios). A roda já vem
              100% dos dados reais do jogo.
            </div>
          </div>
        )}
      </aside>
    </div>
  );
}
