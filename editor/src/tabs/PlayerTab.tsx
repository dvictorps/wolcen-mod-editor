import { useEffect, useState } from "react";
import { api, PlayerStats } from "../api";

type EditMap = Record<string, number>;
const fieldKey = (file: string, element: string, attr: string) =>
  `${file}|${element}|${attr}`;

// Short hints for a few non-obvious stats.
const HINTS: Record<string, string> = {
  StaminaRecoveryTimer:
    "Tempo (s) do cooldown de recuperação da stamina. No Wolcen a stamina recarrega TODAS as cargas após esse timer — diminua para dodges mais frequentes.",
  DodgeBaseDelay: "Atraso base entre dodges (s).",
  StunCooldown: "Tempo mínimo entre stuns sofridos (s).",
  HealthRegenerationBase: "Regeneração de vida por segundo (base).",
};

export default function PlayerTab({
  edits,
  setEdits,
}: {
  edits: EditMap;
  setEdits: (fn: (prev: EditMap) => EditMap) => void;
}) {
  const [data, setData] = useState<PlayerStats | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.getPlayerStats().then(setData).catch((e) => setError(String(e)));
  }, []);

  return (
    <div className="main">
      {error && <div className="error">{error}</div>}
      <h2>Player <span className="muted">(base stats)</span></h2>
      <div className="muted small">{data?.file}</div>
      <div className="perks">
        {data?.groups.map((g) => (
          <div className="perk-card" key={g.element}>
            <div className="perk-head">
              <span className="perk-name">{g.element}</span>
            </div>
            <div className="fields">
              {g.fields.map((f) => {
                const key = fieldKey(data.file, g.element, f.attr);
                const val = key in edits ? edits[key] : f.value;
                const changed = key in edits && edits[key] !== f.value;
                const hint = HINTS[f.attr];
                return (
                  <label className="field" key={f.attr}>
                    <span className="field-name" title={hint || f.attr}>
                      {f.attr}
                      {hint && <span className="hintmark"> ⓘ</span>}
                    </span>
                    <input
                      type="number"
                      step="any"
                      value={val}
                      className={changed ? "changed" : ""}
                      onChange={(e) => {
                        const n = parseFloat(e.target.value);
                        setEdits((prev) => ({ ...prev, [key]: Number.isNaN(n) ? f.value : n }));
                      }}
                    />
                    {changed && <span className="orig">was {f.value}</span>}
                  </label>
                );
              })}
            </div>
            {g.element === "Stamina" && (
              <div className="perk-desc" style={{ marginTop: 8 }}>
                {HINTS.StaminaRecoveryTimer}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
