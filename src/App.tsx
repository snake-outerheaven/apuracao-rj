// SPDX-License-Identifier: GPL-3.0-or-later
import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "./api";
import { EvolutionChart } from "./components/EvolutionChart";
import { MunDetail } from "./components/MunDetail";
import { MunTable } from "./components/MunTable";
import { Regioes } from "./components/Regioes";
import { RjMap } from "./components/RjMap";
import { StatePanel } from "./components/StatePanel";
import { fmtHora } from "./format";
import type { PontoHistorico, Snapshot } from "./types";

type Tema = "claro" | "escuro";
const temaInicial = (): Tema => {
  try {
    const s = localStorage.getItem("tema");
    if (s === "claro" || s === "escuro") return s;
  } catch { /* sem storage */ }
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "escuro" : "claro";
};

export default function App() {
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [hist, setHist] = useState<PontoHistorico[]>([]);
  const [erro, setErro] = useState<string | null>(null);
  const [sel, setSel] = useState<string | null>(null);
  const [tema, setTema] = useState<Tema>(temaInicial);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    document.documentElement.dataset.theme = tema === "escuro" ? "dark" : "light";
    try { localStorage.setItem("tema", tema); } catch { /* ignora */ }
  }, [tema]);

  const recarregaHist = useCallback(() => api.getHistory().then(setHist).catch(() => {}), []);

  useEffect(() => {
    let vivo = true;
    const un: (() => void)[] = [];
    api.getSnapshot().then((s) => vivo && (setSnap(s), setErro(null))).catch((e) => vivo && setErro(String(e)));
    recarregaHist();
    api.onUpdate((s) => { if (vivo) { setSnap(s); setErro(null); recarregaHist(); } }).then((u) => (vivo ? un.push(u) : u()));
    api.onError((m) => vivo && setErro(m)).then((u) => (vivo ? un.push(u) : u()));
    return () => { vivo = false; un.forEach((u) => u()); };
  }, [recarregaHist]);

  const atualiza = async () => {
    setBusy(true);
    try { setSnap(await api.refreshNow()); setErro(null); recarregaHist(); } catch (e) { setErro(String(e)); } finally { setBusy(false); }
  };

  const munSel = useMemo(() => snap?.municipios.find((m) => sel && m.ibge?.slice(0, 6) === sel.slice(0, 6)) ?? null, [snap, sel]);
  const velho = !!snap && (snap.desatualizado || erro !== null);

  return (
    <div className="app">
      <a className="skip" href="#principal">Ir para o conteúdo</a>
      <header>
        <div>
          <h1>Apuração RJ — Presidente, 2º turno 2026</h1>
          {snap && <p className="muted">Atualizado às {fmtHora(snap.atualizadoTse)} (horário de Brasília){snap.fonte === "mock" && " · DADOS FICTÍCIOS (modo demonstração)"}</p>}
        </div>
        <div className="acoes">
          <button type="button" onClick={atualiza} disabled={busy}>{busy ? "Atualizando…" : "Atualizar agora"}</button>
          <button type="button" aria-pressed={tema === "escuro"} onClick={() => setTema(tema === "escuro" ? "claro" : "escuro")}>Tema: {tema === "escuro" ? "escuro" : "claro"}</button>
        </div>
      </header>

      {velho && snap && (
        <div className="banner" role="alert">
          Dados desatualizados desde {fmtHora(snap.atualizadoTse)}.{erro ? ` Falha na última coleta: ${erro}` : ""} Os números abaixo são os últimos recebidos, não necessariamente os atuais.
        </div>
      )}
      {erro && !snap && <div className="banner" role="alert">Não foi possível obter dados: {erro}</div>}

      <main id="principal">
        {!snap && !erro && <p role="status">Carregando…</p>}
        {snap && (
          <>
            <StatePanel estado={snap.estado} geradoEm={snap.geradoEm} atualizadoTse={snap.atualizadoTse} />
            <div className="duo">
              <section className="card" aria-labelledby="h-mapa">
                <h2 id="h-mapa">Mapa por município</h2>
                <RjMap municipios={snap.municipios} selecionado={sel} onSelect={setSel} />
              </section>
              <MunDetail m={munSel} onClose={() => setSel(null)} />
            </div>
            <EvolutionChart historico={hist} estado={snap.estado} />
            <Regioes municipios={snap.municipios} />
            <MunTable municipios={snap.municipios} selecionado={sel} onSelect={setSel} />
          </>
        )}
      </main>

      <footer>
        <strong>App não oficial — dados do TSE.</strong> Fonte: Tribunal Superior Eleitoral (resultados.tse.jus.br); malha municipal: IBGE. Não substitui a divulgação oficial. As cores identificam apenas o número do candidato e não implicam juízo de valor.
      </footer>
    </div>
  );
}
