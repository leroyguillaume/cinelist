import { z } from "zod";

export const sessionSchema = z.object({
  user: z.object({
    id: z.string(),
    username: z.string(),
    isAdmin: z.boolean(),
  }),
});
export type Session = z.infer<typeof sessionSchema>;
export type SessionUser = Session["user"];

export const participantSchema = z.object({
  id: z.string(),
  name: z.string(),
});
export type Participant = z.infer<typeof participantSchema>;

export const filmSchema = z.object({
  id: z.string(),
  title: z.string(),
  addedAt: z.string(),
});
export type Film = z.infer<typeof filmSchema>;

export const phaseSchema = z.enum(["eliminating", "awaitingVerdict", "completed"]);
export type Phase = z.infer<typeof phaseSchema>;

export const listSummarySchema = z.object({
  id: z.string(),
  name: z.string(),
  filmCount: z.number(),
  filmsToAdd: z.number(),
  createdAt: z.string(),
  participants: z.array(participantSchema),
  nextStarterId: z.string().nullable(),
  activeSelection: z
    .object({
      id: z.string(),
      phase: phaseSchema,
      eliminationCount: z.number(),
      eliminationTotal: z.number(),
    })
    .nullable(),
});
export type ListSummary = z.infer<typeof listSummarySchema>;

export const listSchema = listSummarySchema.extend({
  films: z.array(filmSchema),
});
export type List = z.infer<typeof listSchema>;

export const outcomeSchema = z.enum(["watched", "abandoned"]);
export type Outcome = z.infer<typeof outcomeSchema>;

export const eliminationSchema = z.object({
  id: z.string(),
  filmId: z.string().nullable(),
  filmTitle: z.string(),
  participantId: z.string().nullable(),
  participantName: z.string(),
  place: z.number(),
});
export type Elimination = z.infer<typeof eliminationSchema>;

export const verdictSchema = z.object({
  filmTitle: z.string(),
  place: z.number(),
  outcome: outcomeSchema,
  decidedAt: z.string(),
});
export type Verdict = z.infer<typeof verdictSchema>;

export const selectionSchema = z.object({
  id: z.string(),
  listId: z.string(),
  starterName: z.string(),
  filmCount: z.number(),
  startedAt: z.string(),
  completedAt: z.string().nullable(),
  phase: phaseSchema,
  currentParticipant: participantSchema.nullable(),
  candidate: z
    .object({
      filmId: z.string(),
      title: z.string(),
      place: z.number(),
    })
    .nullable(),
  remainingFilms: z.array(z.object({ id: z.string(), title: z.string() })),
  eliminations: z.array(eliminationSchema),
  verdicts: z.array(verdictSchema),
});
export type Selection = z.infer<typeof selectionSchema>;

export const filmStatisticsSchema = z.object({
  film: filmSchema,
  selectionCount: z.number(),
  bestPlace: z.number().nullable(),
  worstPlace: z.number().nullable(),
  averagePlace: z.number().nullable(),
  averageStanding: z.number().nullable(),
  finalsLost: z.number(),
  eliminatedBy: z.array(z.object({ participantName: z.string(), count: z.number() })),
  recentRuns: z.array(
    z.object({
      selectionId: z.string(),
      startedAt: z.string(),
      place: z.number(),
      filmCount: z.number(),
      eliminatedBy: z.string(),
      winnerTitle: z.string().nullable(),
    }),
  ),
});
export type FilmStatistics = z.infer<typeof filmStatisticsSchema>;

export const userStatusSchema = z.enum(["active", "invited", "invitationExpired"]);

export const userSchema = z.object({
  id: z.string(),
  username: z.string(),
  isAdmin: z.boolean(),
  status: userStatusSchema,
  invitationExpiresAt: z.string().nullable(),
  listCount: z.number(),
});
export type User = z.infer<typeof userSchema>;

export const invitationTokenSchema = z.object({
  token: z.string(),
  expiresAt: z.string(),
});
export type InvitationToken = z.infer<typeof invitationTokenSchema>;

export const createdUserSchema = z.object({
  user: userSchema,
  invitation: invitationTokenSchema,
});

export const invitationSchema = z.object({
  username: z.string(),
  expiresAt: z.string(),
});
export type Invitation = z.infer<typeof invitationSchema>;

export function pageSchema<T extends z.ZodType>(item: T) {
  return z.object({
    items: z.array(item),
    page: z.number(),
    perPage: z.number(),
    total: z.number(),
  });
}
export type Page<T> = { items: T[]; page: number; perPage: number; total: number };

export const problemSchema = z.object({
  status: z.number(),
  code: z.string(),
  invalidFields: z.array(z.string()).optional(),
});
