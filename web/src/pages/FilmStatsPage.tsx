import { useTranslation } from "react-i18next";
import { Link, useParams } from "react-router";
import { ErrorText } from "../components/ErrorText";
import { BackIcon } from "../components/Icons";
import { Loading } from "../components/Loading";
import { formatDate, formatPlace } from "../lib/format";
import { useFilmStatistics, useList } from "../lib/queries";
import type { FilmStatistics } from "../lib/schemas";

const TRACK_WIDTH = 520;
const TRACK_PADDING = 10;

/** Horizontal position of a place on a track going from last (left) to first (right). */
export function trackX(place: number, filmCount: number): number {
  const span = Math.max(filmCount - 1, 1);
  return TRACK_PADDING + ((filmCount - place) / span) * (TRACK_WIDTH - 2 * TRACK_PADDING);
}

function Tiles({ stats }: { stats: FilmStatistics }) {
  const { t, i18n } = useTranslation();
  const place = (value: number | null) =>
    value === null ? "–" : formatPlace(value, i18n.language);
  const average =
    stats.averagePlace !== null
      ? {
          value: new Intl.NumberFormat(i18n.language, { maximumFractionDigits: 1 }).format(
            stats.averagePlace,
          ),
          label: t("film.average"),
        }
      : {
          value:
            stats.averageStanding === null
              ? "–"
              : t("film.standingValue", { percent: Math.round((1 - stats.averageStanding) * 100) }),
          label: t("film.standing"),
        };
  const tiles = [
    { value: String(stats.selectionCount), label: t("film.selections"), accent: false },
    { value: place(stats.bestPlace), label: t("film.best"), accent: true },
    { ...average, accent: false },
    { value: place(stats.worstPlace), label: t("film.worst"), accent: false },
    { value: String(stats.finalsLost), label: t("film.finals"), accent: false },
  ];
  return (
    <div className="tiles">
      {tiles.map((tile) => (
        <div key={tile.label} className="tile">
          <span className="tile-value" style={tile.accent ? { color: "var(--accent)" } : undefined}>
            {tile.value}
          </span>
          <span className="tile-label">{tile.label}</span>
        </div>
      ))}
    </div>
  );
}

function Runs({ stats }: { stats: FilmStatistics }) {
  const { t, i18n } = useTranslation();
  const runs = [...stats.recentRuns].reverse();
  return (
    <section className="card stack" style={{ gap: 8 }}>
      <h2>{t("film.runs")}</h2>
      <div className="run-axis" aria-hidden="true">
        <span>{t("film.last")}</span>
        <span>{formatPlace(1, i18n.language)}</span>
      </div>
      {runs.map((run) => {
        const x = trackX(run.place, run.filmCount);
        const label = t("film.runLabel", {
          date: formatDate(run.startedAt, i18n.language),
          place: formatPlace(run.place, i18n.language),
          name: run.eliminatedBy,
        });
        return (
          <div key={run.selectionId} className="run-row">
            <span className="muted" style={{ fontSize: 13 }}>
              {formatDate(run.startedAt, i18n.language)}
            </span>
            <svg
              viewBox={`0 0 ${TRACK_WIDTH} 44`}
              width="100%"
              height="44"
              role="img"
              aria-label={label}
              preserveAspectRatio="none"
            >
              <title>{label}</title>
              <line
                x1={TRACK_PADDING}
                y1="30"
                x2={TRACK_WIDTH - TRACK_PADDING}
                y2="30"
                stroke="var(--line)"
                strokeWidth="2"
                strokeLinecap="round"
                vectorEffect="non-scaling-stroke"
              />
              <line
                x1={x}
                y1="22"
                x2={x}
                y2="38"
                stroke="var(--accent)"
                strokeWidth="10"
                strokeLinecap="round"
                vectorEffect="non-scaling-stroke"
              />
            </svg>
            <div className="run-detail">
              <span>
                {formatPlace(run.place, i18n.language)} · {run.eliminatedBy}
              </span>
              {run.winnerTitle && (
                <span className="muted">{t("film.winner", { title: run.winnerTitle })}</span>
              )}
            </div>
          </div>
        );
      })}
    </section>
  );
}

function EliminatedBy({ stats }: { stats: FilmStatistics }) {
  const { t } = useTranslation();
  return (
    <section className="card stack" style={{ gap: 12 }}>
      <h2>{t("film.eliminatedBy")}</h2>
      {stats.eliminatedBy.map((entry) => (
        <div key={entry.participantName} className="stack" style={{ gap: 6 }}>
          <div className="row" style={{ justifyContent: "space-between", fontSize: 15 }}>
            <span>{entry.participantName}</span>
            <span className="muted">{t("film.times", { count: entry.count })}</span>
          </div>
          <div className="bar-track">
            <div
              className="bar-fill"
              style={{ width: `${(entry.count / stats.selectionCount) * 100}%` }}
            />
          </div>
        </div>
      ))}
    </section>
  );
}

export function FilmStatsPage() {
  const { t, i18n } = useTranslation();
  const { listId = "", filmId = "" } = useParams();
  const list = useList(listId);
  const statistics = useFilmStatistics(filmId);
  const stats = statistics.data;
  return (
    <main className="page">
      <div className="page-header">
        <Link className="icon-button" to={`/lists/${listId}`} aria-label={t("app.back")}>
          <BackIcon />
        </Link>
        <span className="muted spacer">{list.data?.name}</span>
      </div>
      {statistics.isPending && <Loading />}
      <ErrorText error={statistics.error} />
      {stats && (
        <>
          <div className="stack" style={{ gap: 4 }}>
            <h1>{stats.film.title}</h1>
            <span className="muted">
              {t("film.since", { date: formatDate(stats.film.addedAt, i18n.language) })}
            </span>
          </div>
          <Tiles stats={stats} />
          {stats.selectionCount === 0 ? (
            <p className="muted">{t("film.never")}</p>
          ) : (
            <div className="stats-layout stack">
              <Runs stats={stats} />
              <EliminatedBy stats={stats} />
            </div>
          )}
        </>
      )}
    </main>
  );
}
