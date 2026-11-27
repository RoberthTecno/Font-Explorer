import { describe, expect, it } from "vitest";
import { faceCoverage, faceStatus, familyStatus, familyStyles, familyWeights, hasBoldAndItalic, representativeFace, variationById, VARIATIONS, weightLabel } from "./analysis";
import type { CharSetDef, FaceInfo, ScannedFamily } from "../types/scan";

const sets: CharSetDef[] = [
  { id: "lower", label: "a-c", category: "latin", chars: "abc" },
  { id: "enye", label: "ñ", category: "spanish", chars: "ñÑ" },
  { id: "currency", label: "cur", category: "symbols", chars: "€$" },
];
const face = (o: Partial<FaceInfo> = {}): FaceInfo => ({
  faceIndex: 0, subfamilyName: "Regular", fullName: null, weightClass: 400, widthClass: 5, isItalic: false,
  isOblique: false, isVariable: false, axes: [], glyphCount: 100, missing: "", ...o,
});
const fam = (faces: FaceInfo[]): ScannedFamily => ({
  id: "x", familyName: "X", isVariable: false, variantCount: faces.length,
  files: [{ path: "p", fileName: "f", format: "TTF", size: 1, modifiedAt: null, source: "system", faces }],
});

describe("weightLabel", () => {
  it("names standard weights and flags odd ones", () => {
    expect(weightLabel(700)).toBe("Bold");
    expect(weightLabel(450)).toBe("Medium (450)");
  });
});

describe("coverage and status", () => {
  it("lists present and missing characters per set", () => {
    const c = faceCoverage(face({ missing: "ñ€" }), sets);
    expect(c[1].missing).toEqual(["ñ"]);
    expect(c[1].present).toEqual(["Ñ"]);
    expect(c[2].missing).toEqual(["€"]);
  });
  it("is full only when every Latin and Spanish char exists; symbols do not count", () => {
    expect(faceStatus(face({ missing: "€$" }), sets)).toBe("full");
    expect(faceStatus(face({ missing: "ñ" }), sets)).toBe("partial");
    expect(faceStatus(face({ missing: "abcñÑ" }), sets)).toBe("none");
  });
  it("uses the best face of the family", () => {
    expect(familyStatus(fam([face({ missing: "ñ" }), face()]), sets)).toBe("full");
    expect(familyStatus(fam([]), sets)).toBe("unanalyzed");
  });
});

describe("weights and styles", () => {
  it("expands the wght axis of variable fonts", () => {
    const v = face({ isVariable: true, axes: [{ tag: "wght", minValue: 300, defaultValue: 400, maxValue: 700 }] });
    expect(familyWeights(fam([v]))).toEqual([300, 400, 500, 600, 700]);
  });
  it("detects italic and oblique from flags and axes", () => {
    expect(familyStyles(fam([face({ isItalic: true })]))).toEqual({ italic: true, oblique: false });
    const s = face({ axes: [{ tag: "slnt", minValue: -12, defaultValue: 0, maxValue: 0 }] });
    expect(familyStyles(fam([s]))).toEqual({ italic: false, oblique: true });
  });
  it("bold + italic filter", () => {
    expect(hasBoldAndItalic(fam([face({ weightClass: 700 }), face({ isItalic: true })]))).toBe(true);
    expect(hasBoldAndItalic(fam([face()]))).toBe(false);
  });
});

describe("variations (smart filters)", () => {
  const has = (id: string, f: ScannedFamily) => variationById(id)!.test(f);
  it("detects weights, including Ultra Bold (800) and Black (900)", () => {
    const f = fam([face(), face({ weightClass: 800 }), face({ weightClass: 900 })]);
    expect(["w400", "w800", "w900"].map((id) => has(id, f))).toEqual([true, true, true]);
    expect(has("w700", f)).toBe(false);
    expect(has("w100", f)).toBe(false);
  });
  it("detects italic and oblique", () => {
    expect(has("italic", fam([face({ isItalic: true })]))).toBe(true);
    expect(has("oblique", fam([face({ isItalic: true })]))).toBe(false);
    expect(has("oblique", fam([face({ isOblique: true })]))).toBe(true);
  });
  it("detects narrow by width class, wdth axis or style name; wide the same way", () => {
    expect(has("condensed", fam([face({ widthClass: 3 })]))).toBe(true);
    expect(has("condensed", fam([face({ subfamilyName: "Narrow Bold" })]))).toBe(true);
    expect(has("condensed", fam([face({ axes: [{ tag: "wdth", minValue: 75, defaultValue: 100, maxValue: 100 }] })]))).toBe(true);
    expect(has("condensed", fam([face()]))).toBe(false);
    expect(has("expanded", fam([face({ widthClass: 7 })]))).toBe(true);
    expect(has("expanded", fam([face({ widthClass: 3 })]))).toBe(false);
  });
  it("lists each variation once with a unique id", () => {
    expect(new Set(VARIATIONS.map((v) => v.id)).size).toBe(VARIATIONS.length);
  });
});

describe("representative face for card previews", () => {
  const rep = (faces: FaceInfo[]) => representativeFace(fam(faces))?.face.subfamilyName;
  it("prefers Regular over Bold", () => {
    expect(rep([face({ subfamilyName: "Bold", weightClass: 700 }), face({ subfamilyName: "Regular" })])).toBe("Regular");
  });
  it("uses the real weight when a family only has Ultra Bold", () => {
    expect(rep([face({ subfamilyName: "Ultra Bold", weightClass: 800 })])).toBe("Ultra Bold");
  });
  it("prefers an upright face over an italic one and normal width over condensed", () => {
    expect(rep([face({ subfamilyName: "Italic", isItalic: true }), face({ subfamilyName: "Bold", weightClass: 700 })])).toBe("Bold");
    expect(rep([face({ subfamilyName: "Condensed", widthClass: 3 }), face({ subfamilyName: "Light", weightClass: 300 })])).toBe("Light");
  });
  it("returns null for an empty family", () => {
    expect(representativeFace(fam([]))).toBeNull();
  });
});
