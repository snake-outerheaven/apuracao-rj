// SPDX-License-Identifier: GPL-3.0-or-later
import { corCandidato, fmtHora, fmtPct } from "../format";
import type { Abrangencia, PontoHistorico } from "../types";

const W = 640, H = 240, P = { l: 44, r: 12, t: 12, b: 28 };

/** Gráfico SVG próprio: % dos válidos de cada candidato ao longo das coletas. */
export function EvolutionChart({ historico, estado }: { historico: PontoHistorico[]; estado: Abrangencia }) {
  const pts = historico.filter((p) => p.candidatos.some((c) => c.votos > 0));
  return (
    <section className="card" aria-labelledby="h-evo">
      <h2 id="h-evo">Evolução da apuração</h2>
      {pts.length < 2 ? (
        <p className="muted">Aguardando mais coletas para desenhar a evolução.</p>
      ) : (
        <Plot pts={pts} estado={estado} />
      )}
    </section>
  );
}

function Plot({ pts, estado }: { pts: PontoHistorico[]; estado: Abrangencia }) {
  const t0 = Date.parse(pts[0].em), t1 = Date.parse(pts[pts.length - 1].em);
  const span = Math.max(1, t1 - t0);
  const x = (em: string) => P.l + ((Date.parse(em) - t0) / span) * (W - P.l - P.r);
  const y = (v: number) => P.t + (1 - v / 100) * (H - P.t - P.b);
  const pctDe = (p: PontoHistorico, num: string) => {
    const tot = p.candidatos.reduce((s, c) => s + c.votos, 0);
    return tot ? ((p.candidatos.find((c) => c.numero === num)?.votos ?? 0) / tot) * 100 : 0;
  };
  const resumo = estado.candidatos.map((c) => `${c.nome} ${fmtPct(pctDe(pts[pts.length - 1], c.numero))}`).join(", ");
  return (
    <figure>
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={`Evolução do percentual de votos válidos. Último ponto: ${resumo}`}>
        {[0, 25, 50, 75, 100].map((v) => (
          <g key={v}>
            <line x1={P.l} x2={W - P.r} y1={y(v)} y2={y(v)} className="grid" />
            <text x={P.l - 6} y={y(v) + 4} textAnchor="end" className="tick">{v}%</text>
          </g>
        ))}
        <text x={P.l} y={H - 8} className="tick">{fmtHora(pts[0].em)}</text>
        <text x={W - P.r} y={H - 8} textAnchor="end" className="tick">{fmtHora(pts[pts.length - 1].em)}</text>
        {estado.candidatos.map((c) => (
          <polyline key={c.numero} fill="none" stroke={corCandidato(c.numero)} strokeWidth="2.5" points={pts.map((p) => `${x(p.em)},${y(pctDe(p, c.numero))}`).join(" ")} />
        ))}
      </svg>
      <figcaption className="legenda">
        {estado.candidatos.map((c) => (
          <span key={c.numero}><i className="chip" style={{ background: corCandidato(c.numero) }} aria-hidden="true" /> {c.nome}</span>
        ))}
      </figcaption>
    </figure>
  );
}
