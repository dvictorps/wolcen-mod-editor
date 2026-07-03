import { useEffect, useState } from "react";
import { api } from "./api";

export default function SetupScreen({ onReady }: { onReady: () => void }) {
  const [dir, setDir] = useState("");
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [toolsOk, setToolsOk] = useState(true);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api
      .getState()
      .then((s) => {
        setToolsOk(s.tools_ok);
        setDir(s.game_dir || s.detected || "");
        if (s.prepared) onReady();
      })
      .catch((e) => setStatus(String(e)))
      .finally(() => setLoading(false));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function prepare() {
    setBusy(true);
    setStatus("Descriptografando os dados do jogo… (uma vez só, ~10–30s)");
    try {
      await api.setGameDir(dir.trim());
      await api.prepareData();
      setStatus("Pronto!");
      onReady();
    } catch (e) {
      setStatus(`Erro: ${e}`);
    } finally {
      setBusy(false);
    }
  }

  if (loading) {
    return (
      <div className="setup">
        <div className="setup-card">
          <div className="muted">carregando…</div>
        </div>
      </div>
    );
  }

  return (
    <div className="setup">
      <div className="setup-card">
        <h1>Wolcen Mod Editor</h1>
        <p className="muted">
          Primeira execução: aponte a pasta de instalação do Wolcen. O app descriptografa
          os dados necessários uma única vez (fica salvo depois).
        </p>

        {!toolsOk && (
          <div className="error">
            Ferramentas de descriptografia não encontradas no pacote do app.
          </div>
        )}

        <label className="setup-label">Pasta do Wolcen</label>
        <input
          className="setup-input"
          value={dir}
          onChange={(e) => setDir(e.target.value)}
          placeholder={`ex: C:\\Program Files (x86)\\Steam\\steamapps\\common\\Wolcen`}
          spellCheck={false}
        />
        <div className="muted small">
          Cole o caminho da pasta que contém <code>Game\Umbra.pak</code>. Detectamos
          automaticamente se o Wolcen estiver na Steam.
        </div>

        <button className="setup-btn" disabled={busy || !dir.trim()} onClick={prepare}>
          {busy ? "Preparando…" : "Preparar e continuar"}
        </button>

        {status && <div className="setup-status">{status}</div>}

        <div className="muted small setup-foot">
          Precisa de uma cópia legal do Wolcen. Mods são só offline. Se a descriptografia
          falhar, instale o runtime <b>Microsoft Visual C++ 2015–2019 (x86)</b>.
        </div>
      </div>
    </div>
  );
}
