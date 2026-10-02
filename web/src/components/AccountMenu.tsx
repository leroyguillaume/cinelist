import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate } from "react-router";
import { setLanguage } from "../i18n";
import { useSignOut } from "../lib/queries";
import type { SessionUser } from "../lib/schemas";

export function AccountMenu({ user }: { user: SessionUser }) {
  const { t, i18n } = useTranslation();
  const signOut = useSignOut();
  const navigate = useNavigate();
  const menu = useRef<HTMLDetailsElement>(null);
  const close = () => menu.current?.removeAttribute("open");
  return (
    <details className="account-menu" ref={menu}>
      <summary className="avatar" aria-label={t("account.menu")}>
        {user.username.slice(0, 1)}
      </summary>
      <div className="menu">
        <span className="menu-item muted">{user.username}</span>
        {user.isAdmin && (
          <Link className="menu-item" to="/users" onClick={close}>
            {t("account.users")}
          </Link>
        )}
        <button
          type="button"
          className="menu-item"
          onClick={() => {
            setLanguage(i18n.language === "fr" ? "en" : "fr");
            close();
          }}
        >
          {t("account.language")}
        </button>
        <button
          type="button"
          className="menu-item menu-danger"
          onClick={() => signOut.mutate(undefined, { onSettled: () => navigate("/login") })}
        >
          {t("account.signOut")}
        </button>
      </div>
    </details>
  );
}
