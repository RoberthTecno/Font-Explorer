import { describe, expect, it } from "vitest";
import { activeCount, emptyFilters, faceMatchesCoverage, matchesFilters } from "./filters";
import type { Filters, FontFamilyItem } from "../types/font";
import type { CharSetDef, FaceInfo } from "../types/scan";

const sets: CharSetDef[] = [
  { id: "upper", label: "A-B", category: "latin", chars: "AB" },
  { id: "lower", label: "a-b", category: "latin", chars: "ab" },
  { id: "accents", label: "á", category: "spanish", chars: "áÁ" },
  { id: "enye", label: "ñ", category: "spanish", chars: "ñÑ" },
  { id: "currency", label: "cur", category: "symbols", chars: "€$" },
  { id: "arrows", label: "arr", category: "symbols", chars: "←→" },
];
const face = (o: Partial<FaceInfo> = {}): FaceInfo => ({
  faceIndex: 0, subfamilyName: "Regular", fullName: null, weightClass: 400, widthClass: 5, isItalic: false, isOblique: false,
  isVariable: false, axes: [], glyphCount: 1, missing: "", ...o,
});
const item = (id: string, faces: FaceInfo[] | null, fav = false): FontFamilyItem => ({
  id, familyName: id, cssFamily: id, status: "unanalyzed", isFavorite: fav, variantCount: faces?.length ?? 1, isVariable: false,
  scanned: faces && { id, familyName: id, isVariable: false, variantCount: faces.length,
    files: [{ path: id, fileName: id, format: "TTF", size: 1, modifiedAt: null, source: "system", faces }] },
});
const f = (o: Partial<Filters>): Filters => ({ ...emptyFilters(), ...o });
const run = (items: FontFamilyItem[], filters: Filters, tagMap: Record<string, number[]> = {}) =>
  items.filter((i) => matchesFilters(i, filters, sets, tagMap[i.id] ?? [])).map((i) => i.id);

const arial = item("arial", [face(), face({ weightClass: 700, subfamilyName: "Bold" }), face({ isItalic: true })], true);
const noEnye = item("noenye", [face({ missing: "ñÑ" }), face({ weightClass: 700, missing: "ñÑ" })]);
const thin = item("thin", [face({ weightClass: 100, missing: "€$←→" })]);
const sym = item("sym", [face({ missing: "" })]);
const sample = item("sample", null);
const all = [arial, noEnye, thin, sym, sample];

describe("combined filters", () => {
  it("no filters returns everything", () => {
    expect(run(all, f({}))).toHaveLength(5);
    expect(activeCount(f({}))).toBe(0);
  });
  it("combines weight + Spanish coverage with AND", () => {
    expect(run(all, f({ weights: [700] }))).toEqual(["arial", "noenye"]);
    expect(run(all, f({ weights: [700], spanish: true }))).toEqual(["arial"]);
  });
  it("requires every selected weight", () => {
    expect(run(all, f({ weights: [400, 700] }))).toEqual(["arial", "noenye"]);
    expect(run(all, f({ weights: [100, 700] }))).toEqual([]);
  });
  it("styles, accents, symbols and case coverage", () => {
    expect(run(all, f({ italic: true }))).toEqual(["arial"]);
    expect(run(all, f({ oblique: true }))).toEqual([]);
    expect(run(all, f({ accents: true }))).toEqual(["arial", "noenye", "thin", "sym"]);
    expect(run(all, f({ symbols: true }))).toEqual(["arial", "noenye", "sym"]);
    expect(run(all, f({ upper: true, lower: true, spanish: true, symbols: true }))).toEqual(["arial", "sym"]);
  });
  it("favorites and tags combine with analysis filters", () => {
    const tags = { arial: [1, 2], sym: [1], noenye: [2] };
    expect(run(all, f({ favorites: true }))).toEqual(["arial"]);
    expect(run(all, f({ tagIds: [1] }), tags)).toEqual(["arial", "sym"]);
    expect(run(all, f({ tagIds: [1, 2] }), tags)).toEqual(["arial"]);
    expect(run(all, f({ tagIds: [1], weights: [700] }), tags)).toEqual(["arial"]);
  });
  it("never matches unanalyzed fonts on analysis filters, but tag/favorite filters still work", () => {
    expect(run([sample], f({ spanish: true }))).toEqual([]);
    expect(run([sample], f({ tagIds: [9] }), { sample: [9] })).toEqual(["sample"]);
  });
  it("marks the variant that meets the coverage filters", () => {
    const filt = f({ spanish: true });
    expect(faceMatchesCoverage(face(), filt, sets)).toBe(true);
    expect(faceMatchesCoverage(face({ missing: "ñ" }), filt, sets)).toBe(false);
    expect(faceMatchesCoverage(face(), f({}), sets)).toBe(false);
  });
});
