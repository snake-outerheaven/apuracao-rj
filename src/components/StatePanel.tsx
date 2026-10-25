// SPDX-License-Identifier: GPL-3.0-or-later
import { corCandidato, fmtDataHora, fmtInt, fmtPct } from "../format";
import type { Abrangencia } from "../types";

/** Barra de disputa: segmentos proporcionais aos votos válidos, na ordem recebida. */
export function BarraDisputa({ a }: { a: Abrangencia }) {
  const total = a.candidatos.reduce((s, c) => s + c.votos, 0);
  return (
    <div className="barra" role="img" aria-label={a.candidatos.map((c) => `${c.nome} ${fmtPct(c.percentual)}`).join(", ")}>
      {a.candidatos.map((c) => (
        <div key={c.numero} style={{ width: `${total ? (c.votos / total) * 100 : 100 / a.candidatos.length}%`, background: corCandidato(c.numero) }} />
      ))}
    </div>
  );
}

export function Candidatos({ a }: { a: Abrangencia }) {
  return (
    <ul className="cands">
      {a.candidatos.map((c) => (
        <li key={c.numero}>
          <span className="chip" style={{ background: corCandidato(c.numero) }} aria-hidden="true" />
          <span className="cnome">
            <strong>{c.nome}</strong>
            <small>
              {c.numero} · {c.partido}
            </small>
          </span>
          {c.eleito && <span className="selo">Eleito</span>}
          <span className="cvotos">
            <strong>{fmtPct(c.percentual)}</strong>
            <small>{fmtInt(c.votos)} votos</small>
          </span>
        </li>
      ))}
    </ul>
  );
}

export function TotaisApuracao({ a }: { a: Abrangencia }) {
  const t = a.totais;
  return (
    <dl className="totais">
      <div><dt>Seções apuradas</dt><dd>{fmtInt(t.secoesApuradas)} de {fmtInt(t.secoesTotais)} ({fmtPct(t.percentualApurado)})</dd></div>
      <div><dt>Votos válidos</dt><dd>{fmtInt(t.votosValidos)}</dd></div>
      <div><dt>Brancos</dt><dd>{fmtInt(t.brancos)}</dd></div>
      <div><dt>Nulos</dt><dd>{fmtInt(t.nulos)}</dd></div>
      <div><dt>Abstenções</dt><dd>{fmtInt(t.abstencoes)}</dd></div>
      <div><dt>Eleitorado</dt><dd>{fmtInt(t.eleitorado)}</dd></div>
    </dl>
  );
}

export function StatePanel({ estado, geradoEm, atualizadoTse }: { estado: Abrangencia; geradoEm: string; atualizadoTse: string }) {
  return (
    <section className="card" aria-labelledby="h-estado">
      <h2 id="h-estado">{estado.nome} — Presidente, 2º turno</h2>
      <BarraDisputa a={estado} />
      <Candidatos a={estado} />
      <TotaisApuracao a={estado} />
      <p className="muted">
        Última atualização do TSE: {fmtDataHora(atualizadoTse)} · coletado pelo app: {fmtDataHora(geradoEm)}
      </p>
    </section>
  );
}
