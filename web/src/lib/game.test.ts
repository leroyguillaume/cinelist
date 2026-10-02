import { describe, expect, it } from "vitest";
import { eliminationsEach, filmsToAdd } from "./game";

describe("filmsToAdd", () => {
  it("needs nothing when turns are already even", () => {
    expect(filmsToAdd(21, 2)).toBe(0);
    expect(filmsToAdd(22, 3)).toBe(0);
  });

  it("reaches the next count that keeps turns even", () => {
    expect(filmsToAdd(20, 2)).toBe(1);
    expect(filmsToAdd(20, 3)).toBe(2);
  });

  it("wants at least two films", () => {
    expect(filmsToAdd(1, 1)).toBe(1);
    expect(filmsToAdd(1, 2)).toBe(2);
  });
});

describe("eliminationsEach", () => {
  it("splits the eliminations between the participants", () => {
    expect(eliminationsEach(21, 2)).toBe(10);
  });
});
