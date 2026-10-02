import { useTranslation } from "react-i18next";
import { Link, useNavigate } from "react-router";
import { ErrorText } from "../components/ErrorText";
import { PlayIcon, PlusIcon } from "../components/Icons";
import { Loading } from "../components/Loading";
import { useLists, useStartSelection } from "../lib/queries";
import type { ListSummary } from "../lib/schemas";

function ListCard({ list }: { list: ListSummary }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const start = useStartSelection(list.id);
  const missing = list.filmsToAdd;
  const starter = list.participants.find((participant) => participant.id === list.nextStarterId);
  const names = list.participants.map((participant) => participant.name).join(", ");
  const active = list.activeSelection;

  let badge = starter && <span className="badge">{t("lists.starts", { name: starter.name })}</span>;
  if (active) {
    badge = (
      <span className="badge badge-accent">
        {t("lists.running", { done: active.eliminationCount, total: active.eliminationTotal })}
      </span>
    );
  } else if (missing > 0) {
    badge = <span className="badge badge-warn">{t("lists.missing", { count: missing })}</span>;
  }

  return (
    <article className="card list-card">
      <div className="list-card-head">
        <div className="stack" style={{ gap: 4 }}>
          <Link className="list-card-title" to={`/lists/${list.id}`}>
            {list.name}
          </Link>
          <span className="muted" style={{ fontSize: 14 }}>
            {t("lists.films", { count: list.filmCount })} · {names}
          </span>
        </div>
        {badge}
      </div>
      {active ? (
        <Link
          className="button button-accent-outline"
          to={`/lists/${list.id}/selections/${active.id}`}
        >
          {t("lists.resume")}
        </Link>
      ) : missing > 0 ? (
        <Link className="button button-outline" to={`/lists/${list.id}`}>
          {t("lists.complete")}
        </Link>
      ) : (
        <button
          type="button"
          className="button button-primary"
          disabled={start.isPending}
          onClick={() =>
            start.mutate(undefined, {
              onSuccess: (selection) => navigate(`/lists/${list.id}/selections/${selection.id}`),
            })
          }
        >
          <PlayIcon size={18} />
          {t("lists.start")}
        </button>
      )}
      <ErrorText error={start.error} />
    </article>
  );
}

export function ListsPage() {
  const { t } = useTranslation();
  const lists = useLists();
  return (
    <main className="page">
      <div className="page-header">
        <h1>{t("lists.title")}</h1>
        {lists.data && (
          <span className="muted">{t("lists.count", { count: lists.data.total })}</span>
        )}
      </div>
      {lists.isPending && <Loading />}
      <ErrorText error={lists.error} />
      {lists.data?.items.length === 0 && <p className="muted">{t("lists.empty")}</p>}
      <div className="list-cards">
        {lists.data?.items.map((list) => (
          <ListCard key={list.id} list={list} />
        ))}
      </div>
      <Link className="button button-dashed" to="/lists/new">
        <PlusIcon size={18} />
        {t("lists.new")}
      </Link>
    </main>
  );
}
