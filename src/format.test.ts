// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { corCandidato, filtraMunicipios, fmtHora, fmtInt, fmtPct, normaliza } from "./format";
import { mockSnapshotAt } from "./mock";

describe("format", () => {
  it("formata inteiros e percentuais em pt-BR", () => {
    expect(fmtInt(1234567)).toBe("1.234.567");
    expect(fmtPct(51.234)).toBe("51,23%");
  });
  it("formata hora em Brasília", () => {
    expect(fmtHora("2026-10-25T20:05:00Z")).toBe("17:05");
    expect(fmtHora("lixo")).toBe("—");
  });
  it("normaliza acentos e filtra", () => {
    expect(normaliza("  São João ")).toBe("sao joao");
    const s = mockSnapshotAt(3);
    expect(filtraMunicipios(s.municipios, "niteroi").map((m) => m.nome)).toEqual(["Niterói"]);
    expect(filtraMunicipios(s.municipios, "")).toHaveLength(s.municipios.length);
  });
  it("cor é estável por número", () => {
    expect(corCandidato("13")).toBe(corCandidato("13"));
    expect(corCandidato("13")).not.toBe(corCandidato("22"));
  });
});
