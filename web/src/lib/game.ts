export const MAX_PARTICIPANTS = 10;
export const MAX_FILMS = 100;

/** Films to add so that every participant plays as many of the `filmCount - 1` eliminations. */
export function filmsToAdd(filmCount: number, participantCount: number): number {
  if (participantCount < 1) {
    return 0;
  }
  const target = Math.max(filmCount, 2);
  const remainder = (target - 1) % participantCount;
  return (remainder === 0 ? target : target + participantCount - remainder) - filmCount;
}

/** Eliminations each participant plays in a selection over `filmCount` films. */
export function eliminationsEach(filmCount: number, participantCount: number): number {
  return Math.floor((filmCount - 1) / Math.max(participantCount, 1));
}
