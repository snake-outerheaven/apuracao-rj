// SPDX-License-Identifier: GPL-3.0-or-later
// Modelo normalizado — ver docs/CONTRACT.md (idêntico ao serde do backend).
export type Candidato = {
  numero: string;
  nome: string;
  partido: string;
  votos: number;
  /** % dos válidos, 0-100 */
  percentual: number;
  eleito: boolean;
};
export type Totais = {
  secoesTotais: number;
  secoesApuradas: number;
  percentualApurado: number;
  votosValidos: number;
  brancos: number;
  nulos: number;
  abstencoes: number;
  eleitorado: number;
};
export type Abrangencia = {
  codigo: string;
  nome: string;
  ibge?: string;
  totais: Totais;
  candidatos: Candidato[];
};
export type Snapshot = {
  geradoEm: string;
  atualizadoTse: string;
  fonte: "tse" | "mock";
  estado: Abrangencia;
  municipios: Abrangencia[];
  desatualizado: boolean;
};
export type PontoHistorico = {
  em: string;
  percentualApurado: number;
  candidatos: { numero: string; votos: number }[];
};
export type Settings = { pollSeconds: number; mock: boolean };
