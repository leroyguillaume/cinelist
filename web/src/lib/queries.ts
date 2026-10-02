import {
  QueryCache,
  QueryClient,
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { ApiError, request, requestEmpty } from "./api";
import { logger } from "./logger";
import {
  createdUserSchema,
  filmSchema,
  filmStatisticsSchema,
  invitationSchema,
  invitationTokenSchema,
  listSchema,
  listSummarySchema,
  type Outcome,
  pageSchema,
  participantSchema,
  type Session,
  selectionSchema,
  sessionSchema,
  userSchema,
} from "./schemas";

const MAX_PER_PAGE = 100;
const HISTORY_PER_PAGE = 20;

export const keys = {
  session: ["session"] as const,
  lists: ["lists"] as const,
  list: (listId: string) => ["lists", listId] as const,
  history: (listId: string) => ["lists", listId, "history"] as const,
  selection: (selectionId: string) => ["selections", selectionId] as const,
  statistics: (filmId: string) => ["films", filmId, "statistics"] as const,
  users: ["users"] as const,
  invitation: (token: string) => ["invitations", token] as const,
};

export function createQueryClient(): QueryClient {
  const client: QueryClient = new QueryClient({
    queryCache: new QueryCache({
      onError: (error, query) => {
        if (
          error instanceof ApiError &&
          error.status === 401 &&
          query.queryKey[0] !== keys.session[0]
        ) {
          logger.info("session.expired");
          client.setQueryData(keys.session, null);
        }
      },
    }),
    defaultOptions: {
      queries: {
        retry: (failures, error) =>
          !(error instanceof ApiError && error.status < 500) && failures < 2,
        staleTime: 10_000,
      },
    },
  });
  return client;
}

export function useSession() {
  return useQuery({
    queryKey: keys.session,
    queryFn: async (): Promise<Session | null> => {
      try {
        return await request("GET", "/sessions/current", sessionSchema);
      } catch (error) {
        if (error instanceof ApiError && error.status === 401) {
          return null;
        }
        throw error;
      }
    },
  });
}

export function useSignIn() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (credentials: { username: string; password: string }) =>
      request("POST", "/sessions", sessionSchema, credentials),
    onSuccess: (session) => {
      client.clear();
      client.setQueryData(keys.session, session);
    },
  });
}

export function useSignOut() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => requestEmpty("DELETE", "/sessions/current"),
    onSettled: () => {
      client.clear();
      client.setQueryData(keys.session, null);
    },
  });
}

export function useInvitation(token: string) {
  return useQuery({
    queryKey: keys.invitation(token),
    queryFn: () => request("GET", `/invitations/${encodeURIComponent(token)}`, invitationSchema),
  });
}

export function useAcceptInvitation(token: string) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (password: string) =>
      request("PUT", `/invitations/${encodeURIComponent(token)}/acceptance`, sessionSchema, {
        password,
      }),
    onSuccess: (session) => {
      client.clear();
      client.setQueryData(keys.session, session);
    },
  });
}

export function useLists() {
  return useQuery({
    queryKey: keys.lists,
    queryFn: () => request("GET", `/lists?perPage=${MAX_PER_PAGE}`, pageSchema(listSummarySchema)),
  });
}

export function useList(listId: string) {
  return useQuery({
    queryKey: keys.list(listId),
    queryFn: () => request("GET", `/lists/${listId}`, listSchema),
  });
}

/** Refreshes everything derived from one list after a change. */
function useListInvalidation() {
  const client = useQueryClient();
  return (listId: string) =>
    Promise.all([
      client.invalidateQueries({ queryKey: keys.lists, exact: true }),
      client.invalidateQueries({ queryKey: keys.list(listId) }),
    ]);
}

export type NewList = {
  name: string;
  participants: string[];
  films: string[];
};

export function useCreateList() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (list: NewList) => request("POST", "/lists", listSchema, list),
    onSuccess: (list) => {
      client.setQueryData(keys.list(list.id), list);
      return client.invalidateQueries({ queryKey: keys.lists, exact: true });
    },
  });
}

export function useUpdateList(listId: string) {
  const invalidate = useListInvalidation();
  return useMutation({
    mutationFn: (list: { name: string }) => request("PUT", `/lists/${listId}`, listSchema, list),
    onSuccess: () => invalidate(listId),
  });
}

export function useDeleteList(listId: string) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => requestEmpty("DELETE", `/lists/${listId}`),
    onSuccess: () => {
      client.removeQueries({ queryKey: keys.list(listId) });
      return client.invalidateQueries({ queryKey: keys.lists, exact: true });
    },
  });
}

export function useAddParticipant(listId: string) {
  const invalidate = useListInvalidation();
  return useMutation({
    mutationFn: (name: string) =>
      request("POST", `/lists/${listId}/participants`, participantSchema, { name }),
    onSuccess: () => invalidate(listId),
  });
}

export function useRenameParticipant(listId: string) {
  const invalidate = useListInvalidation();
  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) =>
      request("PUT", `/participants/${id}`, participantSchema, { name }),
    onSuccess: () => invalidate(listId),
  });
}

export function useRemoveParticipant(listId: string) {
  const invalidate = useListInvalidation();
  return useMutation({
    mutationFn: (participantId: string) => requestEmpty("DELETE", `/participants/${participantId}`),
    onSuccess: () => invalidate(listId),
  });
}

export function useAddFilm(listId: string) {
  const invalidate = useListInvalidation();
  return useMutation({
    mutationFn: (title: string) => request("POST", `/lists/${listId}/films`, filmSchema, { title }),
    onSuccess: () => invalidate(listId),
  });
}

export function useRenameFilm(listId: string) {
  const invalidate = useListInvalidation();
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ id, title }: { id: string; title: string }) =>
      request("PUT", `/films/${id}`, filmSchema, { title }),
    onSuccess: (film) =>
      Promise.all([
        invalidate(listId),
        client.invalidateQueries({ queryKey: keys.statistics(film.id) }),
      ]),
  });
}

export function useRemoveFilm(listId: string) {
  const invalidate = useListInvalidation();
  return useMutation({
    mutationFn: (filmId: string) => requestEmpty("DELETE", `/films/${filmId}`),
    onSuccess: () => invalidate(listId),
  });
}

export function useFilmStatistics(filmId: string) {
  return useQuery({
    queryKey: keys.statistics(filmId),
    queryFn: () => request("GET", `/films/${filmId}/statistics`, filmStatisticsSchema),
  });
}

export function useHistory(listId: string) {
  return useInfiniteQuery({
    queryKey: keys.history(listId),
    initialPageParam: 1,
    queryFn: ({ pageParam }) =>
      request(
        "GET",
        `/lists/${listId}/selections?page=${pageParam}&perPage=${HISTORY_PER_PAGE}`,
        pageSchema(selectionSchema),
      ),
    getNextPageParam: (last) => (last.page * last.perPage < last.total ? last.page + 1 : undefined),
  });
}

export function useSelection(selectionId: string) {
  return useQuery({
    queryKey: keys.selection(selectionId),
    queryFn: () => request("GET", `/selections/${selectionId}`, selectionSchema),
  });
}

export function useStartSelection(listId: string) {
  const invalidate = useListInvalidation();
  const client = useQueryClient();
  return useMutation({
    mutationFn: () => request("POST", `/lists/${listId}/selections`, selectionSchema),
    onSuccess: (selection) => {
      client.setQueryData(keys.selection(selection.id), selection);
      return invalidate(listId);
    },
  });
}

/** Every step of a selection changes the list it belongs to and its history. */
function useSelectionStep(listId: string, selectionId: string) {
  const invalidate = useListInvalidation();
  const client = useQueryClient();
  return () =>
    Promise.all([
      invalidate(listId),
      client.invalidateQueries({ queryKey: keys.selection(selectionId) }),
      client.invalidateQueries({ queryKey: keys.history(listId) }),
    ]);
}

export function useEliminate(listId: string, selectionId: string) {
  const client = useQueryClient();
  const refresh = useSelectionStep(listId, selectionId);
  return useMutation({
    mutationFn: (filmId: string) =>
      request("POST", `/selections/${selectionId}/eliminations`, selectionSchema, { filmId }),
    onSuccess: (selection) => {
      client.setQueryData(keys.selection(selectionId), selection);
      return refresh();
    },
  });
}

export function useUndoElimination(listId: string, selectionId: string) {
  const refresh = useSelectionStep(listId, selectionId);
  return useMutation({
    mutationFn: (eliminationId: string) =>
      requestEmpty("DELETE", `/selections/${selectionId}/eliminations/${eliminationId}`),
    onSuccess: refresh,
  });
}

export function useJudge(listId: string, selectionId: string) {
  const client = useQueryClient();
  const refresh = useSelectionStep(listId, selectionId);
  return useMutation({
    mutationFn: (verdict: { filmId: string; outcome: Outcome }) =>
      request("POST", `/selections/${selectionId}/verdicts`, selectionSchema, verdict),
    onSuccess: (selection) => {
      client.setQueryData(keys.selection(selectionId), selection);
      return refresh();
    },
  });
}

export function useCancelSelection(listId: string, selectionId: string) {
  const client = useQueryClient();
  const invalidate = useListInvalidation();
  return useMutation({
    mutationFn: () => requestEmpty("DELETE", `/selections/${selectionId}`),
    onSuccess: () => {
      client.removeQueries({ queryKey: keys.selection(selectionId) });
      return invalidate(listId);
    },
  });
}

export function useUsers() {
  return useQuery({
    queryKey: keys.users,
    queryFn: () => request("GET", `/users?perPage=${MAX_PER_PAGE}`, pageSchema(userSchema)),
  });
}

export function useCreateUser() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (username: string) => request("POST", "/users", createdUserSchema, { username }),
    onSuccess: () => client.invalidateQueries({ queryKey: keys.users }),
  });
}

export function useIssueInvitation() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (userId: string) =>
      request("POST", `/users/${userId}/invitations`, invitationTokenSchema),
    onSuccess: () => client.invalidateQueries({ queryKey: keys.users }),
  });
}

export function useDeleteUser() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (userId: string) => requestEmpty("DELETE", `/users/${userId}`),
    onSuccess: () => client.invalidateQueries({ queryKey: keys.users }),
  });
}
