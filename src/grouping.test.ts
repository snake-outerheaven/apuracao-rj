// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { agrupaPorRegiao, lider, ordena, regiaoDe } from "./grouping";
import { mockSnapshotAt } from "./mock";

describe("grouping", () => {
  it("classifica regiões (IBGE 6 ou 7 dígitos)", () => {
    expect(regiaoDe("3304557")).toBe("Capital");
    expect(regiaoDe("330455")).toBe("Capital");
    expect(regiaoDe("3303500")).toBe("Baixada");
    expect(regiaoDe("3303302")).toBe("Interior");
    expect(regiaoDe(undefined)).toBe("Interior");
  });
  it("agrega votos e conserva o total", () => {
    const s = mockSnapshotAt(4);
    const g = agrupaPorRegiao(s.municipios);
    expect(g.map((x) => x.regiao)).toEqual(["Capital", "Baixada", "Interior"]);
    expect(g[0].municipios).toBe(1);
    expect(g[1].municipios).toBe(13);
    expect(g.reduce((a, x) => a + x.municipios, 0)).toBe(92);
    const soma = g.reduce((a, x) => a + x.candidatos.reduce((b, c) => b + c.votos, 0), 0);
    expect(soma).toBe(s.municipios.reduce((a, m) => a + m.candidatos.reduce((b, c) => b + c.votos, 0), 0));
  });
  it("líder e ordenação", () => {
    const s = mockSnapshotAt(4);
    expect(lider({ ...s.municipios[0], candidatos: [] })).toBeNull();
    const o = ordena(s.municipios, { chave: "validos", asc: false });
    expect(o[0].totais.votosValidos).toBeGreaterThanOrEqual(o[1].totais.votosValidos);
    expect(ordena(s.municipios, { chave: "nome", asc: true })[0].nome.localeCompare(o[0].nome, "pt-BR")).toBeLessThanOrEqual(0);
  });
});
