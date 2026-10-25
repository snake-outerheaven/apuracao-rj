// SPDX-License-Identifier: GPL-3.0-or-later
import { useMemo, useState } from "react";
import { municipiosParaCsv } from "../csv";
import { filtraMunicipios, fmtInt, fmtPct } from "../format";
import { ordena, type Ordem } from "../grouping";
import type { Abrangencia } from "../types";

export function baixaCsv(texto: string, nome: string) {
  const url = URL.createObjectURL(new Blob([texto], { type: "text/csv;charset=utf-8" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = nome;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function MunTable({ municipios, selecionado, onSelect }: { municipios: Abrangencia[]; selecionado: string | null; onSelect: (ibge: string) => void }) {
  const [busca, setBusca] = useState("");
  const [ordem, setOrdem] = useState<Ordem>({ chave: "nome", asc: true });
  const cands = municipios[0]?.candidatos ?? [];
  const linhas = useMemo(() => ordena(filtraMunicipios(municipios, busca), ordem), [municipios, busca, ordem]);
  const th = (chave: string, rotulo: string) => {
    const ativo = ordem.chave === chave;
    return (
      <th key={chave} scope="col" aria-sort={ativo ? (ordem.asc ? "ascending" : "descending") : "none"}>
        <button type="button" className="thbtn" onClick={() => setOrdem({ chave, asc: ativo ? !ordem.asc : chave === "nome" })}>
          {rotulo} <span aria-hidden="true">{ativo ? (ordem.asc ? "▲" : "▼") : ""}</span>
        </button>
      </th>
    );
  };
  return (
    <section className="card" aria-labelledby="h-tab">
      <h2 id="h-tab">Municípios ({linhas.length})</h2>
      <div className="toolbar">
        <label>
          <span className="sr">Buscar município</span>
          <input type="search" placeholder="Buscar município…" value={busca} onChange={(e) => setBusca(e.target.value)} />
        </label>
        <button type="button" onClick={() => baixaCsv(municipiosParaCsv(linhas), "apuracao-rj-municipios.csv")}>Exportar CSV</button>
      </div>
      <div className="scroll">
        <table>
          <thead>
            <tr>
              {th("nome", "Município")}
              {th("apurado", "% apurado")}
              {th("validos", "Votos válidos")}
              {cands.map((c) => th(`c:${c.numero}`, `${c.nome} (%)`))}
            </tr>
          </thead>
          <tbody>
            {linhas.map((m) => (
              <tr key={m.codigo} className={selecionado && m.ibge?.slice(0, 6) === selecionado.slice(0, 6) ? "sel" : ""}>
                <th scope="row">
                  <button type="button" className="linkbtn" onClick={() => m.ibge && onSelect(m.ibge)}>{m.nome}</button>
                </th>
                <td>{fmtPct(m.totais.percentualApurado)}</td>
                <td>{fmtInt(m.totais.votosValidos)}</td>
                {cands.map((c) => (
                  <td key={c.numero}>{fmtPct(m.candidatos.find((x) => x.numero === c.numero)?.percentual ?? 0)}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
        {linhas.length === 0 && <p className="muted">Nenhum município encontrado.</p>}
      </div>
    </section>
  );
}
