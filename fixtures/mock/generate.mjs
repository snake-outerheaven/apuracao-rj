// SPDX-License-Identifier: GPL-3.0-or-later
// Gera snapshots mock no formato normalizado (docs/CONTRACT.md).
// Uso: node fixtures/mock/generate.mjs   (determinístico; escreve snapshots/NN.json e sequence.json)
// Candidatos FICTÍCIOS ("Candidato Teste A/B"). Municípios: nomes e códigos reais (TSE e IBGE)
// lidos de fixtures/tse-2026-rj/mun-e006258-cm.rj-only.json (config pública do TSE).
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const cfg = JSON.parse(readFileSync(join(here, "../tse-2026-rj/mun-e006258-cm.rj-only.json"), "utf8"));
const munsCfg = cfg.abr.find((a) => a.cd === "rj").mu;
if (munsCfg.length !== 92) throw new Error("esperados 92 municípios, achei " + munsCfg.length);

// PRNG determinístico (mulberry32)
function rng(seed) {
  return () => {
    seed |= 0; seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const rand = rng(20261025);

const STEPS = [5, 15, 25, 35, 45, 55, 65, 75, 90, 100];
const ELEITORADO_RJ = 12857526;
const CANDS = [
  { numero: "91", nome: "Candidato Teste A", partido: "TESTE-A" },
  { numero: "92", nome: "Candidata Teste B", partido: "TESTE-B" },
];

// Peso por município (a capital pesa ~ 1/4 do estado)
const pesos = munsCfg.map((m) => (m.c === "s" ? 0 : Math.exp(rand() * 2.6) ));
const somaOutros = pesos.reduce((a, b) => a + b, 0);
munsCfg.forEach((m, i) => { if (m.c === "s") pesos[i] = somaOutros * 0.33; });
const somaPesos = pesos.reduce((a, b) => a + b, 0);

// "Final" de cada município (valores 100% apurados)
const finais = munsCfg.map((m, i) => {
  const eleitorado = Math.max(2000, Math.round((pesos[i] / somaPesos) * ELEITORADO_RJ));
  const secoes = Math.max(3, Math.round(eleitorado / 340));
  const comparecimento = 0.70 + rand() * 0.12;
  const comp = Math.round(eleitorado * comparecimento);
  const brancos = Math.round(comp * (0.012 + rand() * 0.012));
  const nulos = Math.round(comp * (0.025 + rand() * 0.02));
  const validos = comp - brancos - nulos;
  const shareA = m.c === "s" ? 0.42 : Math.min(0.72, Math.max(0.3, 0.55 + (rand() - 0.5) * 0.4));
  const votosA = Math.round(validos * shareA);
  // ritmo de apuração próprio (municípios pequenos terminam antes)
  const ritmo = m.c === "s" ? 0.8 : 0.95 + rand() * 0.4;
  return { m, eleitorado, secoes, comp, brancos, nulos, validos, votosA, votosB: validos - votosA, ritmo };
});

// fração apurada do município no passo p (monótona, =1 no passo final)
function fracao(f, p) {
  if (p >= 100) return 1;
  const x = p / 100;
  return Math.min(1, x * f.ritmo);
}
const r2 = (n) => Math.round(n * 100) / 100;

function abrangencia(codigo, nome, ibge, parts) {
  // parts: [{secoes, apuradas, eleitorado, abst, brancos, nulos, votosA, votosB}] já escalados
  const soma = (k) => parts.reduce((a, p) => a + p[k], 0);
  const validos = soma("votosA") + soma("votosB");
  const ord = [
    { ...CANDS[0], votos: soma("votosA") },
    { ...CANDS[1], votos: soma("votosB") },
  ].sort((a, b) => b.votos - a.votos);
  const apur = soma("apuradas"), tot = soma("secoes");
  const out = {
    codigo, nome,
    ...(ibge ? { ibge } : {}),
    totais: {
      secoesTotais: tot, secoesApuradas: apur,
      percentualApurado: tot ? r2((apur / tot) * 100) : 0,
      votosValidos: validos, brancos: soma("brancos"), nulos: soma("nulos"),
      abstencoes: soma("abst"), eleitorado: soma("eleitorado"),
    },
    candidatos: ord.map((c) => ({
      numero: c.numero, nome: c.nome, partido: c.partido, votos: c.votos,
      percentual: validos ? r2((c.votos / validos) * 100) : 0, eleito: false,
    })),
  };
  return out;
}

mkdirSync(join(here, "snapshots"), { recursive: true });
const t0 = Date.UTC(2026, 9, 25, 20, 0, 0); // 17:00 BRT
const seq = [];
STEPS.forEach((p, idx) => {
  const munsOut = [];
  const todas = [];
  finais.forEach((f) => {
    const fr = fracao(f, p);
    const apuradas = Math.round(f.secoes * fr);
    const k = f.secoes ? apuradas / f.secoes : 0;
    const part = {
      secoes: f.secoes, apuradas, eleitorado: f.eleitorado,
      abst: Math.round((f.eleitorado - f.comp) * k),
      brancos: Math.round(f.brancos * k), nulos: Math.round(f.nulos * k),
      votosA: Math.round(f.votosA * k), votosB: Math.round(f.votosB * k),
    };
    todas.push(part);
    munsOut.push(abrangencia(f.m.cd, titulo(f.m.nm), f.m.cdi, [part]));
  });
  const estado = abrangencia("RJ", "Rio de Janeiro", undefined, todas);
  if (p >= 100) {
    for (const a of [estado, ...munsOut]) if (a.candidatos[0].votos > a.candidatos[1].votos) a.candidatos[0].eleito = a === estado;
  }
  const quando = new Date(t0 + idx * 15 * 60000).toISOString();
  seq.push({ geradoEm: quando, atualizadoTse: quando, fonte: "mock", estado, municipios: munsOut, desatualizado: false });
});

function titulo(s) {
  const peq = new Set(["DE", "DA", "DO", "DAS", "DOS", "E"]);
  return s.toLowerCase().split(" ").map((w, i) => (i > 0 && peq.has(w.toUpperCase()) ? w : w.charAt(0).toUpperCase() + w.slice(1)))
    .join(" ").replace(/(-)(\p{L})/gu, (_, h, c) => h + c.toUpperCase());
}

seq.forEach((s, i) => writeFileSync(join(here, "snapshots", String(i + 1).padStart(2, "0") + ".json"), JSON.stringify(s, null, 2) + "\n"));
writeFileSync(join(here, "sequence.json"), JSON.stringify(seq) + "\n");
console.log("ok:", seq.length, "snapshots;", seq.map((s) => s.estado.totais.percentualApurado + "%").join(" "));
