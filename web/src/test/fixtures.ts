import type { Selection } from "../lib/schemas";

export function selection(overrides: Partial<Selection> = {}): Selection {
  return {
    id: "selection-1",
    listId: "list-1",
    starterName: "Sam",
    filmCount: 3,
    startedAt: "2026-09-27T20:00:00Z",
    completedAt: null,
    phase: "eliminating",
    currentParticipant: { id: "alex", name: "Alex" },
    candidate: null,
    remainingFilms: [
      { id: "heat", title: "Heat" },
      { id: "drive", title: "Drive" },
      { id: "alien", title: "Alien" },
    ],
    eliminations: [],
    verdicts: [],
    ...overrides,
  };
}
