import { useTranslation } from "react-i18next";
import { Link, NavLink } from "react-router";
import type { List } from "../lib/schemas";
import { BackIcon, SettingsIcon } from "./Icons";

export function ListHeader({ list }: { list: List }) {
  const { t } = useTranslation();
  return (
    <>
      <div className="page-header">
        <Link className="icon-button page-back" to="/" aria-label={t("app.back")}>
          <BackIcon />
        </Link>
        <h1>{list.name}</h1>
        <Link
          className="icon-button"
          to={`/lists/${list.id}/settings`}
          aria-label={t("list.settings")}
        >
          <SettingsIcon />
        </Link>
      </div>
      <nav className="tabs">
        <NavLink className="tab" to={`/lists/${list.id}`} end>
          {t("list.tabFilms", { count: list.filmCount })}
        </NavLink>
        <NavLink className="tab" to={`/lists/${list.id}/history`}>
          {t("list.tabHistory")}
        </NavLink>
      </nav>
    </>
  );
}

export function TurnOrder({ list }: { list: List }) {
  const { t } = useTranslation();
  return (
    <div className="row" style={{ flexWrap: "wrap" }}>
      <span className="muted" style={{ fontSize: 13 }}>
        {t("list.turnOrder")}
      </span>
      {list.participants.map((participant, index) => (
        <span
          key={participant.id}
          className={`chip${participant.id === list.nextStarterId ? " chip-active" : ""}`}
        >
          <span className="chip-number">{index + 1}</span>
          {participant.name}
        </span>
      ))}
    </div>
  );
}
