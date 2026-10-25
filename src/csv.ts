// SPDX-License-Identifier: GPL-3.0-or-later
import type { Abrangencia } from "./types";

/** Escapa um campo CSV (separador ';' para abrir direto no Excel pt-BR). */
export function csvCampo(v: string | number): string {
  const s = String(v);
  return /[";\n\r]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
}

const num = (n: number) => String(n).replace(".", ",");

export function municipiosParaCsv(lista: Abrangencia[]): string {
  const numeros: { numero: string; nome: string }[] = [];
  for (const m of lista) for (const c of m.candidatos) if (!numeros.some((x) => x.numero === c.numero)) numeros.push({ numero: c.numero, nome: c.nome });
  const cab = ["codigo", "ibge", "municipio", "secoes_totais", "secoes_apuradas", "percentual_apurado", "votos_validos", "brancos", "nulos", "abstencoes", "eleitorado"];
  for (const c of numeros) cab.push(`${c.numero} ${c.nome} votos`, `${c.numero} ${c.nome} %`);
  const linhas = [cab.map(csvCampo).join(";")];
  for (const m of lista) {
    const t = m.totais;
    const row: (string | number)[] = [m.codigo, m.ibge ?? "", m.nome, t.secoesTotais, t.secoesApuradas, num(t.percentualApurado), t.votosValidos, t.brancos, t.nulos, t.abstencoes, t.eleitorado];
    for (const c of numeros) {
      const x = m.candidatos.find((k) => k.numero === c.numero);
      row.push(x ? x.votos : "", x ? num(x.percentual) : "");
    }
    linhas.push(row.map(csvCampo).join(";"));
  }
  return "﻿" + linhas.join("\r\n") + "\r\n";
}
