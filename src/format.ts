// SPDX-License-Identifier: GPL-3.0-or-later
import type { Abrangencia } from "./types";

const nf = new Intl.NumberFormat("pt-BR");
const pf = new Intl.NumberFormat("pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });

export const fmtInt = (n: number): string => nf.format(Math.round(n));
/** Percentual 0-100 com vírgula decimal. */
export const fmtPct = (n: number): string => `${pf.format(n)}%`;

/** HH:MM no fuso de Brasília (America/Sao_Paulo). Retorna "—" se inválido. */
export function fmtHora(iso: string, timeZone = "America/Sao_Paulo"): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  return new Intl.DateTimeFormat("pt-BR", { hour: "2-digit", minute: "2-digit", hour12: false, timeZone }).format(d);
}

export function fmtDataHora(iso: string, timeZone = "America/Sao_Paulo"): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  return new Intl.DateTimeFormat("pt-BR", { dateStyle: "short", timeStyle: "short", timeZone }).format(d);
}

/** Normaliza para busca: minúsculas, sem acentos. */
export function normaliza(s: string): string {
  return s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase().trim();
}

export const filtraMunicipios = (lista: Abrangencia[], termo: string): Abrangencia[] => {
  const t = normaliza(termo);
  return t ? lista.filter((m) => normaliza(m.nome).includes(t)) : lista;
};

/** Cor fixa por número de candidato (hash determinístico em paleta daltônico-amigável). Sem juízo de valor. */
const PALETA = ["#0072b2", "#d55e00", "#009e73", "#cc79a7", "#e69f00", "#56b4e9", "#8c564b", "#7f7f7f"];
export function corCandidato(numero: string): string {
  const n = parseInt(numero, 10);
  const i = Number.isFinite(n) ? n : [...numero].reduce((a, c) => a + c.charCodeAt(0), 0);
  return PALETA[Math.abs(i) % PALETA.length];
}
