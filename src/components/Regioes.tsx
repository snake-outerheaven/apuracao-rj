// SPDX-License-Identifier: GPL-3.0-or-later
import { useMemo } from "react";
import { fmtInt, fmtPct } from "../format";
import { agrupaPorRegiao } from "../grouping";
import type { Abrangencia } from "../types";
import { BarraDisputa } from "./StatePanel";

export function Regioes({ municipios }: { municipios: Abrangencia[] }) {
  const grupos = useMemo(() => agrupaPorRegiao(municipios), [municipios]);
  return (
    <section className="card" aria-labelledby="h-reg">
      <h2 id="h-reg">Capital, Baixada e interior</h2>
      <div className="regioes">
        {grupos.map((g) => {
          const a: Abrangencia = { codigo: g.regiao, nome: g.regiao, totais: { secoesTotais: g.secoesTotais, secoesApuradas: g.secoesApuradas, percentualApurado: g.secoesTotais ? (g.secoesApuradas / g.secoesTotais) * 100 : 0, votosValidos: g.votosValidos, brancos: 0, nulos: 0, abstencoes: 0, eleitorado: 0 }, candidatos: g.candidatos };
          return (
            <div key={g.regiao}>
              <h3>{g.regiao} <small className="muted">({g.municipios} {g.municipios === 1 ? "município" : "municípios"})</small></h3>
              <BarraDisputa a={a} />
              <ul className="mini">
                {g.candidatos.map((c) => <li key={c.numero}>{c.nome}: <strong>{fmtPct(c.percentual)}</strong> ({fmtInt(c.votos)})</li>)}
              </ul>
              <p className="muted">{fmtPct(a.totais.percentualApurado)} das seções apuradas</p>
            </div>
          );
        })}
      </div>
    </section>
  );
}
