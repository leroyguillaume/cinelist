import { useTranslation } from "react-i18next";
import { Link, useNavigate, useParams } from "react-router";
import { ErrorText } from "../components/ErrorText";
import { CheckIcon, CloseIcon, UndoIcon } from "../components/Icons";
import { Splash } from "../components/Loading";
import { Segments } from "../components/Segments";
import { elides, formatPlace } from "../lib/format";
import {
  useCancelSelection,
  useEliminate,
  useJudge,
  useList,
  useSelection,
  useUndoElimination,
} from "../lib/queries";
import type { Selection } from "../lib/schemas";

type ViewProps = { listId: string; selection: Selection };

function EliminatedPanel({ selection }: { selection: Selection }) {
  const { t, i18n } = useTranslation();
  return (
    <section className="eliminated-panel" aria-live="polite">
      <h2 className="muted" style={{ fontSize: 13, fontFamily: "var(--font-body)" }}>
        {t("selection.eliminated", { count: selection.eliminations.length })}
      </h2>
      {selection.eliminations.map((elimination) => (
        <div key={elimination.id} className="ranking-row">
          <span className="ranking-place">{formatPlace(elimination.place, i18n.language)}</span>
          <span className="ranking-title ranking-struck">{elimination.filmTitle}</span>
          <span className="muted">{elimination.participantName}</span>
        </div>
      ))}
    </section>
  );
}

export function EliminationView({ listId, selection }: ViewProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const eliminate = useEliminate(listId, selection.id);
  const cancel = useCancelSelection(listId, selection.id);
  const player = selection.currentParticipant?.name ?? "";
  const total = selection.filmCount - 1;
  const done = selection.eliminations.length;
  return (
    <div className="selection-layout">
      <div className="stack" style={{ gap: 18 }}>
        <div className="stack" style={{ gap: 8 }}>
          <h1 className="turn-title">
            {elides(player)
              ? t("selection.turn", { name: player, context: "elided" })
              : t("selection.turn", { name: player })}
          </h1>
          <p className="muted">
            {t("selection.step", { step: done + 1, total })} · {t("selection.hint")}
          </p>
          <Segments
            total={total}
            done={done}
            showCurrent
            label={t("selection.step", { step: done + 1, total })}
          />
        </div>
        <div className="film-grid">
          {selection.remainingFilms.map((film) => (
            <button
              key={film.id}
              type="button"
              className="film-tile"
              disabled={eliminate.isPending}
              onClick={() => eliminate.mutate(film.id)}
            >
              {film.title}
            </button>
          ))}
        </div>
        <ErrorText error={eliminate.error} />
      </div>
      <div className="stack">
        <EliminatedPanel selection={selection} />
        <button
          type="button"
          className="link-button"
          disabled={cancel.isPending}
          onClick={() => {
            if (window.confirm(t("selection.confirmCancel"))) {
              cancel.mutate(undefined, { onSuccess: () => navigate(`/lists/${listId}`) });
            }
          }}
        >
          {t("selection.cancel")}
        </button>
        <ErrorText error={cancel.error} />
      </div>
    </div>
  );
}

export function VerdictView({ listId, selection }: ViewProps) {
  const { t } = useTranslation();
  const judge = useJudge(listId, selection.id);
  const candidate = selection.candidate;
  if (!candidate) {
    return null;
  }
  const finalist = selection.eliminations.find((elimination) => elimination.place === 2);
  const previous = selection.verdicts.at(-1);
  const next = selection.eliminations.find(
    (elimination) => elimination.place === candidate.place + 1,
  );
  return (
    <div className="elected">
      <div className="elected-frame">
        <span className="elected-label">{t("elected.label")}</span>
        <h1 className="elected-title">{candidate.title}</h1>
      </div>
      <div className="verdict-actions">
        <p className="muted" style={{ fontSize: 17, lineHeight: 1.5 }}>
          {candidate.place === 1 ? (
            <>
              {t("elected.survivor", { count: selection.filmCount })}{" "}
              {finalist &&
                t("elected.final", { name: finalist.participantName, title: finalist.filmTitle })}
            </>
          ) : (
            previous && t("elected.replacement", { title: previous.filmTitle })
          )}
        </p>
        <h2 style={{ fontSize: 22, marginTop: 12 }}>{t("elected.question")}</h2>
        <button
          type="button"
          className="button button-primary"
          disabled={judge.isPending}
          onClick={() => judge.mutate({ filmId: candidate.filmId, outcome: "watched" })}
        >
          <CheckIcon size={20} />
          {t("elected.watched")}
        </button>
        <button
          type="button"
          className="button button-outline verdict-secondary"
          disabled={judge.isPending}
          onClick={() => judge.mutate({ filmId: candidate.filmId, outcome: "abandoned" })}
        >
          <span>{t("elected.abandoned")}</span>
          <span className="muted" style={{ fontSize: 13, fontWeight: 400 }}>
            {next ? t("elected.next", { title: next.filmTitle }) : t("elected.noNext")}
          </span>
        </button>
        <ErrorText error={judge.error} />
        <Link className="link-button" style={{ alignSelf: "center" }} to={`/lists/${listId}`}>
          {t("elected.later")}
        </Link>
      </div>
    </div>
  );
}

function CompletedView({ listId, selection }: ViewProps) {
  const { t } = useTranslation();
  const watched = selection.verdicts.find((verdict) => verdict.outcome === "watched");
  return (
    <div className="elected">
      <div className="stack" style={{ alignItems: "center", textAlign: "center" }}>
        <h1>{t("completed.title")}</h1>
        <p className="muted">
          {watched ? t("completed.watched", { title: watched.filmTitle }) : t("completed.nothing")}
        </p>
        <Link className="button button-primary" to={`/lists/${listId}`}>
          {t("completed.back")}
        </Link>
        <Link className="button button-outline" to={`/lists/${listId}/history`}>
          {t("completed.history")}
        </Link>
      </div>
    </div>
  );
}

function UndoButton({ listId, selection }: ViewProps) {
  const { t } = useTranslation();
  const undo = useUndoElimination(listId, selection.id);
  const latest = selection.eliminations[0];
  const enabled =
    latest !== undefined && selection.verdicts.length === 0 && selection.phase !== "completed";
  return (
    <button
      type="button"
      className="icon-button"
      aria-label={t("selection.undo")}
      title={t("selection.undo")}
      disabled={!enabled || undo.isPending}
      onClick={() => latest && undo.mutate(latest.id)}
    >
      <UndoIcon />
    </button>
  );
}

export function SelectionPage() {
  const { t } = useTranslation();
  const { listId = "", selectionId = "" } = useParams();
  const list = useList(listId);
  const selection = useSelection(selectionId);
  if (selection.isPending) {
    return <Splash />;
  }
  return (
    <div className="focus-screen">
      <header className="focus-bar">
        <Link className="icon-button" to={`/lists/${listId}`} aria-label={t("selection.quit")}>
          <CloseIcon />
        </Link>
        <span className="focus-bar-title">{list.data?.name}</span>
        {selection.data ? (
          <UndoButton listId={listId} selection={selection.data} />
        ) : (
          <span style={{ width: 44 }} />
        )}
      </header>
      <main className="focus-body">
        <ErrorText error={selection.error} />
        {selection.data?.phase === "eliminating" && (
          <EliminationView listId={listId} selection={selection.data} />
        )}
        {selection.data?.phase === "awaitingVerdict" && (
          <VerdictView listId={listId} selection={selection.data} />
        )}
        {selection.data?.phase === "completed" && (
          <CompletedView listId={listId} selection={selection.data} />
        )}
      </main>
    </div>
  );
}
