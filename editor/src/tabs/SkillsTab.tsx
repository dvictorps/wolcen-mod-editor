import { useEffect, useMemo, useState } from "react";
import { api, SkillSummary, SkillDetail } from "../api";

type EditMap = Record<string, number>;

function fieldKey(file: string, uid: string, element: string, attr: string) {
  return `${file}|${uid}|${element}|${attr}`;
}

type DisabledMap = Record<string, boolean>;

export default function SkillsTab({
  edits,
  setEdits,
  disabled,
  setDisabled,
}: {
  edits: EditMap;
  setEdits: (fn: (prev: EditMap) => EditMap) => void;
  disabled: DisabledMap;
  setDisabled: (fn: (prev: DisabledMap) => DisabledMap) => void;
}) {
  const [skills, setSkills] = useState<SkillSummary[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [detail, setDetail] = useState<SkillDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    api
      .listSkills()
      .then((s) => {
        setSkills(s);
        // Preselect Bleeding Edge (our anchor) if present.
        const bleed = s.find((x) => x.internal_name === "Laceration");
        if (bleed) select(bleed.internal_name);
      })
      .catch((e) => setError(String(e)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function select(name: string) {
    setSelected(name);
    setLoading(true);
    setError(null);
    api
      .getSkill(name)
      .then((d) => setDetail(d))
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }

  const editableVariants = useMemo(
    () => (detail ? detail.variants.filter((v) => v.fields.length > 0) : []),
    [detail]
  );

  return (
    <div className="split">
      <aside className="rail">
        <div className="rail-title">Skills ({skills.length})</div>
        {skills.map((s) => (
          <button
            key={s.internal_name}
            className={"rail-item" + (s.internal_name === selected ? " active" : "")}
            onClick={() => select(s.internal_name)}
            title={s.internal_name}
          >
            {s.display_name}
          </button>
        ))}
      </aside>

      <section className="main">
        {error && <div className="error">{error}</div>}
        {detail && (
          <>
            <h2>
              {detail.display_name}{" "}
              <span className="muted">({detail.internal_name})</span>
            </h2>
            <div className="muted small">
              {editableVariants.length} editable perks · {detail.file}
            </div>
            {loading && <div className="muted">loading…</div>}
            <div className="perks">
              {editableVariants.map((v) => (
                <div className="perk-card" key={v.uid}>
                  <div className="perk-head">
                    <span className="perk-num">#{v.number ?? "base"}</span>
                    <span className="perk-name">{v.name || v.uid}</span>
                  </div>
                  {v.description && <div className="perk-desc">{v.description}</div>}
                  <div className="fields">
                    {v.fields.map((f) => {
                      const key = fieldKey(detail.file, v.uid, f.element, f.attr);
                      const off = !!disabled[key];
                      const val = key in edits ? edits[key] : f.value;
                      const changed = key in edits && edits[key] !== f.value;
                      return (
                        <div className={"field" + (off ? " off" : "")} key={key}>
                          <input
                            type="checkbox"
                            className="mod-toggle"
                            checked={!off}
                            title={
                              off
                                ? "modificador desativado — marque para reativar"
                                : "desmarque para desativar este modificador"
                            }
                            onChange={() =>
                              setDisabled((prev) => ({ ...prev, [key]: !off }))
                            }
                          />
                          <span className="field-name" title={f.element}>
                            {f.attr}
                          </span>
                          <input
                            type="number"
                            step="any"
                            value={val}
                            disabled={off}
                            className={changed && !off ? "changed" : ""}
                            onChange={(e) => {
                              const n = parseFloat(e.target.value);
                              setEdits((prev) => ({
                                ...prev,
                                [key]: Number.isNaN(n) ? f.value : n,
                              }));
                            }}
                          />
                          {off ? (
                            <span className="orig off-tag">removido</span>
                          ) : changed ? (
                            <span className="orig" title="valor original">
                              was {f.value}
                            </span>
                          ) : null}
                        </div>
                      );
                    })}
                  </div>
                </div>
              ))}
            </div>
          </>
        )}
      </section>
    </div>
  );
}
