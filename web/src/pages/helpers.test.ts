import { describe, expect, it } from "vitest";
import { trackX } from "./FilmStatsPage";
import { parseTitles } from "./NewListPage";
import { invitationLink } from "./UsersPage";

describe("parseTitles", () => {
  it("keeps one trimmed title per non-empty line", () => {
    expect(parseTitles(" Heat \n\nDrive\n  \n")).toEqual(["Heat", "Drive"]);
  });
});

describe("invitationLink", () => {
  it("points at the invitation page of this origin", () => {
    expect(invitationLink("a+b", "https://cinelist.example")).toBe(
      "https://cinelist.example/invitation/a%2Bb",
    );
  });
});

describe("trackX", () => {
  it("puts the last place on the left and the first on the right", () => {
    expect(trackX(21, 21)).toBeLessThan(trackX(2, 21));
    expect(trackX(1, 21)).toBe(510);
  });
});
