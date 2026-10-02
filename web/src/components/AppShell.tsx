import { useTranslation } from "react-i18next";
import { Link, NavLink, Outlet } from "react-router";
import { useLists, useSession } from "../lib/queries";
import { AccountMenu } from "./AccountMenu";
import { PlusIcon, UsersIcon } from "./Icons";

export function AppShell() {
  const { t } = useTranslation();
  const session = useSession();
  const lists = useLists();
  const user = session.data?.user;
  if (!user) {
    return null;
  }
  return (
    <div className="shell">
      <aside className="sidebar">
        <Link to="/">
          <img className="sidebar-logo" src="/logo-title.svg" alt="CineList" />
        </Link>
        <span className="sidebar-heading">{t("lists.title")}</span>
        <nav className="stack" style={{ gap: 4 }}>
          {lists.data?.items.map((list) => (
            <NavLink key={list.id} to={`/lists/${list.id}`} className="sidebar-link">
              {list.name}
              {list.activeSelection ? (
                <span className="badge badge-accent">{t("history.running")}</span>
              ) : (
                <span className="muted" style={{ fontSize: 13 }}>
                  {list.filmCount}
                </span>
              )}
            </NavLink>
          ))}
        </nav>
        <Link
          to="/lists/new"
          className="button button-dashed button-small"
          style={{ marginTop: 8 }}
        >
          <PlusIcon size={16} />
          {t("lists.new")}
        </Link>
        <div className="spacer" />
        {user.isAdmin && (
          <NavLink to="/users" className="sidebar-link" style={{ justifyContent: "flex-start" }}>
            <UsersIcon size={18} />
            {t("account.users")}
          </NavLink>
        )}
        <div className="sidebar-account">
          <span className="spacer">{user.username}</span>
          <AccountMenu user={user} />
        </div>
      </aside>
      <div className="shell-main">
        <header className="topbar">
          <Link to="/">
            <img className="topbar-logo" src="/logo-title.svg" alt="CineList" />
          </Link>
          <AccountMenu user={user} />
        </header>
        <Outlet />
      </div>
    </div>
  );
}
