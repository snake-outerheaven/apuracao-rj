// SPDX-License-Identifier: GPL-3.0-or-later
import type { Abrangencia, Candidato } from "./types";

export type Regiao = "Capital" | "Baixada" | "Interior";
export const REGIOES: Regiao[] = ["Capital", "Baixada", "Interior"];

const CAPITAL = "330455";
/** Baixada Fluminense (13 municípios, códigos IBGE de 6 dígitos). */
const BAIXADA = new Set(["330045", "330170", "330185", "330200", "330227", "330250", "330285", "330320", "330350", "330360", "330414", "330510", "330555"]);

/** Aceita IBGE com 6 ou 7 dígitos. */
export function regiaoDe(ibge: string | undefined): Regiao {
  const c = (ibge ?? "").slice(0, 6);
  if (c === CAPITAL) return "Capital";
  return BAIXADA.has(c) ? "Baixada" : "Interior";
}

export type GrupoAgregado = {
  regiao: Regiao;
  municipios: number;
  secoesTotais: number;
  secoesApuradas: number;
  votosValidos: number;
  candidatos: Candidato[];
};

/** Soma votos por região; percentual recalculado sobre votos válidos agregados. Ordem dos candidatos = ordem do primeiro município. */
export function agrupaPorRegiao(municipios: Abrangencia[]): GrupoAgregado[] {
  return REGIOES.map((regiao) => {
    const ms = municipios.filter((m) => regiaoDe(m.ibge) === regiao);
    const mapa = new Map<string, Candidato>();
    let validos = 0, st = 0, sa = 0;
    for (const m of ms) {
      validos += m.totais.votosValidos;
      st += m.totais.secoesTotais;
      sa += m.totais.secoesApuradas;
      for (const c of m.candidatos) {
        const x = mapa.get(c.numero);
        if (x) x.votos += c.votos;
        else mapa.set(c.numero, { ...c, votos: c.votos, eleito: false, percentual: 0 });
      }
    }
    const candidatos = [...mapa.values()].map((c) => ({ ...c, percentual: validos > 0 ? (c.votos / validos) * 100 : 0 }));
    return { regiao, municipios: ms.length, secoesTotais: st, secoesApuradas: sa, votosValidos: validos, candidatos };
  });
}

/** Candidato com mais votos (null se empate no topo ou sem votos) — para colorir o mapa. */
export function lider(a: Abrangencia): Candidato | null {
  const [p, s] = [...a.candidatos].sort((x, y) => y.votos - x.votos);
  if (!p || p.votos === 0) return null;
  return s && s.votos === p.votos ? null : p;
}

export type Ordem = { chave: string; asc: boolean };
/** Ordena municípios por nome, apurado, votos válidos, ou por número de candidato (chave "c:13"). */
export function ordena(lista: Abrangencia[], { chave, asc }: Ordem): Abrangencia[] {
  const val = (m: Abrangencia): number | string => {
    if (chave === "nome") return m.nome;
    if (chave === "apurado") return m.totais.percentualApurado;
    if (chave === "validos") return m.totais.votosValidos;
    if (chave.startsWith("c:")) return m.candidatos.find((c) => c.numero === chave.slice(2))?.percentual ?? -1;
    return m.nome;
  };
  const dir = asc ? 1 : -1;
  return [...lista].sort((a, b) => {
    const x = val(a), y = val(b);
    const r = typeof x === "string" ? x.localeCompare(String(y), "pt-BR") : (x as number) - (y as number);
    return r * dir || a.nome.localeCompare(b.nome, "pt-BR");
  });
}
