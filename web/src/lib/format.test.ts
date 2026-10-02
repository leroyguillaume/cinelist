import { describe, expect, it } from "vitest";
import { elides, formatPlace } from "./format";

describe("formatPlace", () => {
  it("writes French ordinals", () => {
    expect(formatPlace(1, "fr")).toBe("1er");
    expect(formatPlace(21, "fr")).toBe("21e");
  });

  it("writes English ordinals", () => {
    expect(formatPlace(1, "en")).toBe("1st");
    expect(formatPlace(2, "en")).toBe("2nd");
    expect(formatPlace(3, "en")).toBe("3rd");
    expect(formatPlace(11, "en")).toBe("11th");
    expect(formatPlace(21, "en")).toBe("21st");
  });
});

describe("elides", () => {
  it("elides before a vowel", () => {
    expect(elides("Alex")).toBe(true);
    expect(elides("Élodie")).toBe(true);
    expect(elides("Sam")).toBe(false);
  });
});
