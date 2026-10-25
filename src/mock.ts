// SPDX-License-Identifier: GPL-3.0-or-later
// Gerador de snapshots fictícios (fonte "mock") para o navegador/dev. Dados INVENTADOS.
import { MUNICIPIOS_MOCK } from "./mockMunicipios";
import type { Abrangencia, PontoHistorico, Snapshot } from "./types";

/** PRNG determinístico (mulberry32). */
function rng(seed: number) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const CANDS = [
  { numero: "10", nome: "Candidato A (fictício)", partido: "AAA" },
  { numero: "20", nome: "Candidata B (fictícia)", partido: "BBB" },
];
export const PASSOS_MOCK = 12;
const BASE = Date.parse("2026-10-25T20:00:00Z");

function abr(codigo: string, nome: string, ibge: string | undefined, eleitorado: number, fracao: number, afinidade: number, seed: number): Abrangencia {
  const r = rng(seed);
  const secoesTotais = Math.max(5, Math.round(eleitorado / 300));
  const secoesApuradas = Math.min(secoesTotais, Math.round(secoesTotais * fracao));
  const f = secoesApuradas / secoesTotais;
  const comparecimento = 0.78 + r() * 0.08;
  const votantes = Math.round(eleitorado * f * comparecimento);
  const brancos = Math.round(votantes * 0.012);
  const nulos = Math.round(votantes * 0.03);
  const validos = votantes - brancos - nulos;
  const vA = Math.round(validos * afinidade);
  const vB = validos - vA;
  const votos = [vA, vB];
  const candidatos = CANDS.map((c, i) => ({ ...c, votos: votos[i], percentual: validos ? (votos[i] / validos) * 100 : 0, eleito: false })).sort((a, b) => b.votos - a.votos);
  return {
    codigo, nome, ibge,
    totais: { secoesTotais, secoesApuradas, percentualApurado: f * 100, votosValidos: validos, brancos, nulos, abstencoes: Math.round(eleitorado * f) - votantes, eleitorado },
    candidatos,
  };
}

/** Snapshot do passo `n` (0..PASSOS_MOCK); a apuração avança com n. */
export function mockSnapshotAt(n: number): Snapshot {
  const passo = Math.max(0, Math.min(PASSOS_MOCK, n));
  const global = passo / PASSOS_MOCK;
  const municipios = MUNICIPIOS_MOCK.map(([ibge, nome, mil], i) => {
    const r = rng(i * 131 + 7);
    const afin = 0.3 + r() * 0.4;
    const ritmo = 0.6 + r() * 0.8;
    const fr = Math.min(1, global * ritmo);
    return abr(String(10000 + i), nome, ibge, mil * 1000, fr, afin, i * 977 + 3);
  });
  const soma = <K extends keyof Abrangencia["totais"]>(k: K) => municipios.reduce((a, m) => a + m.totais[k], 0);
  const votosPor = (num: string) => municipios.reduce((a, m) => a + (m.candidatos.find((c) => c.numero === num)?.votos ?? 0), 0);
  const validos = soma("votosValidos");
  const st = soma("secoesTotais"), sa = soma("secoesApuradas");
  let candidatos = CANDS.map((c) => ({ ...c, votos: votosPor(c.numero), percentual: 0, eleito: false }));
  candidatos.forEach((c) => (c.percentual = validos ? (c.votos / validos) * 100 : 0));
  candidatos = candidatos.sort((a, b) => b.votos - a.votos);
  if (passo === PASSOS_MOCK) candidatos[0].eleito = true; // só no fim da apuração fictícia
  const estado: Abrangencia = {
    codigo: "RJ", nome: "Rio de Janeiro",
    totais: { secoesTotais: st, secoesApuradas: sa, percentualApurado: st ? (sa / st) * 100 : 0, votosValidos: validos, brancos: soma("brancos"), nulos: soma("nulos"), abstencoes: soma("abstencoes"), eleitorado: soma("eleitorado") },
    candidatos,
  };
  const quando = new Date(BASE + passo * 5 * 60000).toISOString();
  return { geradoEm: quando, atualizadoTse: quando, fonte: "mock", estado, municipios, desatualizado: false };
}

export function mockHistorico(ate: number): PontoHistorico[] {
  const out: PontoHistorico[] = [];
  for (let i = 1; i <= Math.max(1, ate); i++) {
    const s = mockSnapshotAt(i);
    out.push({ em: s.geradoEm, percentualApurado: s.estado.totais.percentualApurado, candidatos: s.estado.candidatos.map((c) => ({ numero: c.numero, votos: c.votos })) });
  }
  return out;
}
