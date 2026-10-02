import type { Selection } from "./schemas";

export type RankingEntry = {
  place: number;
  title: string;
  /** Who eliminated the film; absent for the film that survived every elimination. */
  eliminatedBy: string | null;
};

/** Every place of a selection, best first. */
export function ranking(selection: Selection): RankingEntry[] {
  const eliminated: RankingEntry[] = selection.eliminations.map((elimination) => ({
    place: elimination.place,
    title: elimination.filmTitle,
    eliminatedBy: elimination.participantName,
  }));
  const survivor =
    selection.verdicts.find((verdict) => verdict.place === 1)?.filmTitle ??
    (selection.phase === "awaitingVerdict" && selection.candidate?.place === 1
      ? selection.candidate.title
      : null);
  const entries = survivor
    ? [{ place: 1, title: survivor, eliminatedBy: null }, ...eliminated]
    : eliminated;
  return entries.sort((a, b) => a.place - b.place);
}
