import { useTranslation } from "react-i18next";
import { useParams } from "react-router";
import { ErrorText } from "../components/ErrorText";
import { ListHeader } from "../components/ListHeader";
import { Loading } from "../components/Loading";
import { formatDate, formatPlace } from "../lib/format";
import { useHistory, useList } from "../lib/queries";
import { ranking } from "../lib/ranking";
import type { Selection } from "../lib/schemas";

function SelectionCard({ selection }: { selection: Selection }) {
  const { t, i18n } = useTranslation();
  const watched = selection.verdicts.find((verdict) => verdict.outcome === "watched");
  const abandoned = selection.verdicts.filter((verdict) => verdict.outcome === "abandoned");
  const running = selection.phase !== "completed";
  const entries = ranking(selection);
  return (
    <article className="card stack" style={{ gap: 10 }}>
      <div className="row" style={{ justifyContent: "space-between" }}>
        <span className="muted" style={{ fontSize: 13 }}>
          {t("history.started", {
            date: formatDate(selection.startedAt, i18n.language),
            name: selection.starterName,
          })}
        </span>
        {running ? (
          <span className="badge">{t("history.running")}</span>
        ) : watched ? (
          <span className="badge badge-accent">{t("history.watched")}</span>
        ) : (
          <span className="badge badge-muted">{t("history.abandoned")}</span>
        )}
      </div>
      {!running && <h2>{watched?.filmTitle ?? t("history.nothing")}</h2>}
      {abandoned.map((verdict) => (
        <p key={verdict.place} className="muted" style={{ fontSize: 14 }}>
          <span style={{ textDecoration: "line-through" }}>{verdict.filmTitle}</span>{" "}
          {t("history.abandonedLine")}
        </p>
      ))}
      {entries.length > 0 && (
        <details>
          <summary className="link-button" style={{ color: "var(--accent)", padding: "8px 0" }}>
            {t("history.positions", { count: selection.filmCount })}
          </summary>
          <ol
            className="stack"
            style={{ gap: 8, listStyle: "none", margin: "8px 0 0", padding: 0 }}
          >
            {entries.map((entry) => (
              <li key={entry.place} className="ranking-row">
                <span className="ranking-place">{formatPlace(entry.place, i18n.language)}</span>
                <span className="ranking-title">{entry.title}</span>
                <span className="muted">{entry.eliminatedBy ?? t("history.elected")}</span>
              </li>
            ))}
          </ol>
        </details>
      )}
    </article>
  );
}

export function HistoryPage() {
  const { t } = useTranslation();
  const { listId = "" } = useParams();
  const list = useList(listId);
  const history = useHistory(listId);
  const selections = history.data?.pages.flatMap((page) => page.items) ?? [];
  return (
    <main className="page">
      {list.data && <ListHeader list={list.data} />}
      {(list.isPending || history.isPending) && <Loading />}
      <ErrorText error={list.error ?? history.error} />
      {history.data && selections.length === 0 && <p className="muted">{t("history.empty")}</p>}
      <div className="list-cards">
        {selections.map((selection) => (
          <SelectionCard key={selection.id} selection={selection} />
        ))}
      </div>
      {history.hasNextPage && (
        <button
          type="button"
          className="button button-outline"
          disabled={history.isFetchingNextPage}
          onClick={() => void history.fetchNextPage()}
        >
          {t("app.loadMore")}
        </button>
      )}
    </main>
  );
}
