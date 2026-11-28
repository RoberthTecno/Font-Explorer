import { describe, expect, it } from "vitest";
import { compareLayout } from "./compareLayout";
import { SAMPLE_EN, SAMPLE_ES } from "../data/sampleTexts";
import { previewChips } from "../data/mockFonts";

describe("comparison layout", () => {
  it("uses columns for up to 3 fonts", () => {
    expect(compareLayout(1)).toEqual({ kind: "columns", cols: 1, rows: 1 });
    expect(compareLayout(2)).toEqual({ kind: "columns", cols: 2, rows: 1 });
    expect(compareLayout(3)).toEqual({ kind: "columns", cols: 3, rows: 1 });
  });
  it("switches to tiles from 4 fonts", () => {
    expect(compareLayout(4)).toEqual({ kind: "tiles", cols: 2, rows: 2 });
    expect(compareLayout(5)).toEqual({ kind: "tiles", cols: 3, rows: 2 });
    expect(compareLayout(6)).toEqual({ kind: "tiles", cols: 3, rows: 2 });
  });
  it("never lays out fewer than one cell", () => {
    expect(compareLayout(0).cols).toBe(1);
  });
});

describe("sample texts", () => {
  it("are the full sentences", () => {
    expect(SAMPLE_EN).toBe("The quick brown fox jumps over the lazy dog and runs away.");
    expect(SAMPLE_ES).toBe("Benjamín pidió una bebida de kiwi y fresa. Noé, sin vergüenza, la más exquisita champaña del menú");
  });
  it("are what the preview bar chips insert", () => {
    expect(previewChips.find((c) => c.label === "fox")?.text).toBe(SAMPLE_EN);
    expect(previewChips.find((c) => c.label === "español")?.text).toBe(SAMPLE_ES);
  });
});
