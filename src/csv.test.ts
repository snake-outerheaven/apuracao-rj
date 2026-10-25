// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { csvCampo, municipiosParaCsv } from "./csv";
import { mockSnapshotAt } from "./mock";

describe("csv", () => {
  it("escapa campos", () => {
    expect(csvCampo('a;b')).toBe('"a;b"');
    expect(csvCampo('diz "oi"')).toBe('"diz ""oi"""');
    expect(csvCampo(5)).toBe("5");
  });
  it("gera cabeçalho e uma linha por município, com vírgula decimal", () => {
    const s = mockSnapshotAt(2);
    const linhas = municipiosParaCsv(s.municipios).replace("﻿", "").trim().split("\r\n");
    expect(linhas).toHaveLength(s.municipios.length + 1);
    expect(linhas[0].startsWith("codigo;ibge;municipio")).toBe(true);
    expect(linhas[1]).toMatch(/\d,\d/);
  });
});
