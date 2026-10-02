import { describe, expect, it } from "vitest";
import { selection } from "../test/fixtures";
import { ranking } from "./ranking";

describe("ranking", () => {
  it("puts the survivor first and the first film out last", () => {
    const entries = ranking(
      selection({
        phase: "completed",
        eliminations: [
          {
            id: "e2",
            filmId: "drive",
            filmTitle: "Drive",
            participantId: "sam",
            participantName: "Sam",
            place: 2,
          },
          {
            id: "e1",
            filmId: "heat",
            filmTitle: "Heat",
            participantId: "alex",
            participantName: "Alex",
            place: 3,
          },
        ],
        verdicts: [
          { filmTitle: "Alien", place: 1, outcome: "watched", decidedAt: "2026-09-27T22:00:00Z" },
        ],
      }),
    );
    expect(entries.map((entry) => entry.title)).toEqual(["Alien", "Drive", "Heat"]);
    expect(entries[0]?.eliminatedBy).toBeNull();
  });
});
