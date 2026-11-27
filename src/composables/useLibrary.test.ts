import { beforeEach, describe, expect, it } from "vitest";
import { useLibrary } from "./useLibrary";
import { emptyFilters, resultsLabel } from "../services/filters";

// Fuera de Tauri la biblioteca usa los 28 datos de ejemplo: sirven para comprobar el conteo y la selección.
const lib = useLibrary();
beforeEach(() => {
  lib.search.value = "";
  lib.section.value = { kind: "all" };
  lib.filters.value = emptyFilters();
  lib.clearCompare();
});

describe("Showing X fonts out of N", () => {
  it("counts the whole library when nothing is applied", () => {
    expect(lib.totalFonts.value).toBe(28);
    expect(lib.visibleFonts.value.length).toBe(28);
    expect(resultsLabel(28, 28)).toBe("Showing 28 fonts out of 28");
  });
  it("uses the singular for one font", () => {
    expect(resultsLabel(1, 28)).toBe("Showing 1 font out of 28");
  });
  it("keeps the total fixed while search, section, tags, favorites and filters narrow the result", () => {
    lib.search.value = "arial";
    expect(lib.visibleFonts.value.map((f) => f.familyName)).toEqual(["Arial"]);
    expect(lib.totalFonts.value).toBe(28);

    lib.search.value = "";
    lib.section.value = { kind: "favorites" };
    const favs = lib.visibleFonts.value.length;
    expect(favs).toBeGreaterThan(0);
    expect(favs).toBeLessThan(28);

    lib.section.value = { kind: "all" };
    const tag = lib.tags.value.find((t) => t.name === "Gótica")!;
    lib.filters.value = { ...emptyFilters(), tagIds: [tag.id] };
    const tagged = lib.visibleFonts.value.map((f) => f.familyName);
    expect(tagged).toEqual(expect.arrayContaining(["Franklin Gothic Medium", "Old English Text MT"]));
    expect(tagged.length).toBe(2);

    lib.filters.value = { ...emptyFilters(), tagIds: [tag.id], favorites: true };
    expect(lib.visibleFonts.value.length).toBe(0); // ninguna gótica es favorita
    expect(lib.totalFonts.value).toBe(28);
  });
});

describe("compare selection", () => {
  it("toggles fonts, caps at the maximum and clears", () => {
    const ids = lib.fonts.value.map((f) => f.id);
    expect(lib.toggleCompare(ids[0])).toBe(true);
    expect(lib.compareIds.value).toEqual([ids[0]]);
    lib.toggleCompare(ids[0]);
    expect(lib.compareIds.value).toEqual([]);

    for (const id of ids.slice(0, lib.MAX_COMPARE)) lib.toggleCompare(id);
    expect(lib.compareIds.value.length).toBe(lib.MAX_COMPARE);
    expect(lib.toggleCompare(ids[lib.MAX_COMPARE])).toBe(false);
    expect(lib.compareIds.value.length).toBe(lib.MAX_COMPARE);
    expect(lib.compareNotice.value).toContain("up to 6");

    lib.clearCompare();
    expect(lib.compareIds.value).toEqual([]);
  });
  it("only opens the comparison with at least two fonts", () => {
    lib.toggleCompare(lib.fonts.value[0].id);
    lib.openCompare();
    expect(lib.compareOpen.value).toBe(false);
    lib.toggleCompare(lib.fonts.value[1].id);
    lib.openCompare();
    expect(lib.compareOpen.value).toBe(true);
  });
});

describe("export targets", () => {
  it("never offers sample (unanalyzed) fonts", () => {
    expect(lib.exportTargets("shown")).toEqual([]);
  });
  it("'shown' follows search and filters; 'selected' follows the circle selection", () => {
    const original = lib.fonts.value;
    const fake = (id: string) => ({ ...original[0], id, familyName: id, scanned: { id, familyName: id, isVariable: false, variantCount: 1, files: [] } });
    lib.fonts.value = [fake("aaa"), fake("bbb"), fake("ccc")];
    expect(lib.exportTargets("shown")).toEqual(["aaa", "bbb", "ccc"]);
    lib.search.value = "bb";
    expect(lib.exportTargets("shown")).toEqual(["bbb"]);
    lib.search.value = "";
    lib.toggleCompare("ccc");
    lib.toggleCompare("zzz"); // no existe en la biblioteca: se ignora al exportar
    expect(lib.exportTargets("selected")).toEqual(["ccc"]);
    lib.fonts.value = original;
  });
});
