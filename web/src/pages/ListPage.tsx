import { type FormEvent, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useParams } from "react-router";
import { ErrorText } from "../components/ErrorText";
import { ChartIcon, MoreIcon } from "../components/Icons";
import { ListHeader, TurnOrder } from "../components/ListHeader";
import { Loading } from "../components/Loading";
import { formatDate } from "../lib/format";
import { eliminationsEach, MAX_FILMS } from "../lib/game";
import {
  useAddFilm,
  useHistory,
  useList,
  useRemoveFilm,
  useRenameFilm,
  useStartSelection,
} from "../lib/queries";
import type { Film, List } from "../lib/schemas";

function FilmRow({ list, film, index }: { list: List; film: Film; index: number }) {
  const { t } = useTranslation();
  const rename = useRenameFilm(list.id);
  const remove = useRemoveFilm(list.id);
  const menu = useRef<HTMLDetailsElement>(null);
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState(film.title);
  const frozen = list.activeSelection !== null;

  const submit = (event: FormEvent) => {
    event.preventDefault();
    rename.mutate({ id: film.id, title }, { onSuccess: () => setEditing(false) });
  };

  return (
    <li className="film-row">
      <span className="film-row-number">{index + 1}</span>
      {editing ? (
        <form className="inline-form" onSubmit={submit}>
          <input
            className="input"
            aria-label={t("list.filmTitle")}
            required
            maxLength={200}
            // biome-ignore lint/a11y/noAutofocus: the field appears because the user asked to edit it.
            autoFocus
            value={title}
            onChange={(event) => setTitle(event.target.value)}
          />
          <button type="submit" className="button button-primary button-small">
            {t("app.save")}
          </button>
          <button
            type="button"
            className="button button-outline button-small"
            onClick={() => setEditing(false)}
          >
            {t("app.cancel")}
          </button>
        </form>
      ) : (
        <Link className="film-row-title" to={`/lists/${list.id}/films/${film.id}`}>
          {film.title}
        </Link>
      )}
      {!editing && (
        <Link
          className="icon-button film-row-stats"
          to={`/lists/${list.id}/films/${film.id}`}
          aria-label={t("list.statisticsOf", { title: film.title })}
          title={t("list.statistics")}
        >
          <ChartIcon size={18} />
        </Link>
      )}
      {!editing && (
        <details className="account-menu" ref={menu}>
          <summary className="icon-button" aria-label={t("app.options")}>
            <MoreIcon />
          </summary>
          <div className="menu">
            <Link className="menu-item" to={`/lists/${list.id}/films/${film.id}`}>
              {t("list.statistics")}
            </Link>
            <button
              type="button"
              className="menu-item"
              onClick={() => {
                menu.current?.removeAttribute("open");
                setTitle(film.title);
                setEditing(true);
              }}
            >
              {t("app.rename")}
            </button>
            {!frozen && (
              <button
                type="button"
                className="menu-item menu-danger"
                onClick={() => {
                  menu.current?.removeAttribute("open");
                  if (window.confirm(t("list.confirmRemoveFilm", { title: film.title }))) {
                    remove.mutate(film.id);
                  }
                }}
              >
                {t("app.remove")}
              </button>
            )}
          </div>
        </details>
      )}
      <ErrorText error={rename.error ?? remove.error} />
    </li>
  );
}

function AddFilmForm({ list }: { list: List }) {
  const { t } = useTranslation();
  const add = useAddFilm(list.id);
  const [title, setTitle] = useState("");
  const missing = list.filmsToAdd;

  const submit = (event: FormEvent) => {
    event.preventDefault();
    add.mutate(title.trim(), { onSuccess: () => setTitle("") });
  };

  return (
    <section className={missing > 0 ? "notice stack" : "stack"} style={{ gap: 10 }}>
      {missing > 0 && <p style={{ fontWeight: 600 }}>{t("list.missing", { count: missing })}</p>}
      <form className="inline-form" onSubmit={submit}>
        <label htmlFor="new-film" className="visually-hidden">
          {t("list.filmTitle")}
        </label>
        <input
          id="new-film"
          className="input"
          required
          maxLength={200}
          placeholder={t("list.filmTitle")}
          value={title}
          onChange={(event) => setTitle(event.target.value)}
        />
        <button type="submit" className="button button-primary" disabled={add.isPending}>
          {t("app.add")}
        </button>
      </form>
      <ErrorText error={add.error} />
    </section>
  );
}

function StartButton({ list }: { list: List }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const start = useStartSelection(list.id);
  const missing = list.filmsToAdd;
  if (list.activeSelection) {
    return (
      <Link
        className="button button-accent-outline button-block"
        to={`/lists/${list.id}/selections/${list.activeSelection.id}`}
      >
        {t("list.resume")}
      </Link>
    );
  }
  return (
    <>
      <button
        type="button"
        className="button button-primary button-block"
        disabled={missing > 0 || start.isPending}
        onClick={() =>
          start.mutate(undefined, {
            onSuccess: (selection) => navigate(`/lists/${list.id}/selections/${selection.id}`),
          })
        }
      >
        {missing > 0 ? t("list.stillMissing", { count: missing }) : t("list.start")}
      </button>
      <ErrorText error={start.error} />
    </>
  );
}

function ListAside({ list }: { list: List }) {
  const { t, i18n } = useTranslation();
  const history = useHistory(list.id);
  const last = history.data?.pages[0]?.items[0];
  const winner = last?.verdicts.find((verdict) => verdict.outcome === "watched");
  return (
    <aside className="list-aside">
      {list.filmsToAdd === 0 && (
        <section className="card stack" style={{ gap: 10 }}>
          <h2>{t("list.turns")}</h2>
          <p className="muted">
            {t("list.turnsEach", {
              count: eliminationsEach(list.filmCount, list.participants.length),
            })}
          </p>
        </section>
      )}
      {last && (
        <Link
          className="card stack"
          style={{ gap: 6, textDecoration: "none", color: "inherit" }}
          to={`/lists/${list.id}/history`}
        >
          <span className="muted" style={{ fontSize: 13 }}>
            {t("list.lastSelection", { date: formatDate(last.startedAt, i18n.language) })}
          </span>
          <h2>{winner?.filmTitle ?? t("history.running")}</h2>
          <span style={{ color: "var(--accent)", fontWeight: 600, fontSize: 14 }}>
            {t("list.seeHistory")}
          </span>
        </Link>
      )}
    </aside>
  );
}

export function ListPage() {
  const { t } = useTranslation();
  const { listId = "" } = useParams();
  const list = useList(listId);
  if (list.isPending) {
    return (
      <main className="page">
        <Loading />
      </main>
    );
  }
  if (list.isError) {
    return (
      <main className="page">
        <ErrorText error={list.error} />
      </main>
    );
  }
  const data = list.data;
  return (
    <main className="page">
      <ListHeader list={data} />
      <div className="list-layout">
        <div className="stack">
          <TurnOrder list={data} />
          {data.activeSelection && <p className="notice">{t("list.frozen")}</p>}
          {!data.activeSelection && data.filmCount < MAX_FILMS && <AddFilmForm list={data} />}
          <ul className="film-list">
            {data.films.map((film, index) => (
              <FilmRow key={film.id} list={data} film={film} index={index} />
            ))}
          </ul>
          <div className="sticky-footer">
            <StartButton list={data} />
          </div>
        </div>
        <ListAside list={data} />
      </div>
    </main>
  );
}
