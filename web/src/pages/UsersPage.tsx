import { type FormEvent, useState } from "react";
import { useTranslation } from "react-i18next";
import { ErrorText } from "../components/ErrorText";
import { Loading } from "../components/Loading";
import { formatDate } from "../lib/format";
import { logger } from "../lib/logger";
import { useCreateUser, useDeleteUser, useIssueInvitation, useUsers } from "../lib/queries";
import type { InvitationToken, User } from "../lib/schemas";

type IssuedInvitation = { username: string; invitation: InvitationToken };

export function invitationLink(token: string, origin: string = window.location.origin): string {
  return `${origin}/invitation/${encodeURIComponent(token)}`;
}

function InvitationBox({ issued }: { issued: IssuedInvitation }) {
  const { t, i18n } = useTranslation();
  const [copied, setCopied] = useState(false);
  const link = invitationLink(issued.invitation.token);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(link);
      setCopied(true);
    } catch (error) {
      logger.warn("clipboard.write_failed", { error: String(error) });
    }
  };
  return (
    <section className="card card-highlight invite-box" aria-live="polite">
      <p>
        {t("users.created", {
          username: issued.username,
          date: formatDate(issued.invitation.expiresAt, i18n.language),
        })}
      </p>
      <div className="copy-row">
        <label htmlFor="invitation-link" className="visually-hidden">
          {t("users.link")}
        </label>
        <input
          id="invitation-link"
          className="input"
          readOnly
          value={link}
          onFocus={(event) => event.target.select()}
        />
        <button type="button" className="button button-accent-outline" onClick={() => void copy()}>
          {copied ? t("users.copied") : t("users.copy")}
        </button>
      </div>
    </section>
  );
}

function StatusBadge({ user }: { user: User }) {
  const { t, i18n } = useTranslation();
  if (user.isAdmin) {
    return <span className="badge">{t("users.admin")}</span>;
  }
  if (user.status === "invited" && user.invitationExpiresAt) {
    return (
      <span className="badge badge-warn">
        {t("users.invited", { date: formatDate(user.invitationExpiresAt, i18n.language) })}
      </span>
    );
  }
  if (user.status === "invitationExpired") {
    return <span className="badge badge-muted">{t("users.invitationExpired")}</span>;
  }
  return <span className="badge badge-ok">{t("users.active")}</span>;
}

function UserCard({
  user,
  onIssued,
}: {
  user: User;
  onIssued: (issued: IssuedInvitation) => void;
}) {
  const { t } = useTranslation();
  const issue = useIssueInvitation();
  const remove = useDeleteUser();
  return (
    <article className="card user-card">
      <div className="row" style={{ flexWrap: "wrap" }}>
        <h2 className="spacer">{user.username}</h2>
        <StatusBadge user={user} />
      </div>
      <span className="muted" style={{ fontSize: 14 }}>
        {t("users.lists", { count: user.listCount })}
      </span>
      {user.isAdmin ? (
        <span className="hint">{t("users.managed")}</span>
      ) : (
        <div className="row" style={{ flexWrap: "wrap" }}>
          <button
            type="button"
            className="button button-outline button-small"
            disabled={issue.isPending}
            onClick={() =>
              issue.mutate(user.id, {
                onSuccess: (invitation) => onIssued({ username: user.username, invitation }),
              })
            }
          >
            {user.status === "active" ? t("users.reset") : t("users.newLink")}
          </button>
          <button
            type="button"
            className="button button-danger button-small"
            disabled={remove.isPending}
            onClick={() => {
              if (window.confirm(t("users.confirmDelete", { username: user.username }))) {
                remove.mutate(user.id);
              }
            }}
          >
            {t("app.delete")}
          </button>
        </div>
      )}
      <ErrorText error={issue.error ?? remove.error} />
    </article>
  );
}

export function UsersPage() {
  const { t } = useTranslation();
  const users = useUsers();
  const create = useCreateUser();
  const [username, setUsername] = useState("");
  const [issued, setIssued] = useState<IssuedInvitation | null>(null);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    create.mutate(username.trim(), {
      onSuccess: (created) => {
        setIssued({ username: created.user.username, invitation: created.invitation });
        setUsername("");
      },
    });
  };

  return (
    <main className="page">
      <div className="stack" style={{ gap: 4 }}>
        <h1>{t("users.title")}</h1>
        <p className="muted">{t("users.intro")}</p>
      </div>
      <form className="inline-form" onSubmit={submit}>
        <label htmlFor="new-user" className="visually-hidden">
          {t("users.username")}
        </label>
        <input
          id="new-user"
          className="input"
          required
          minLength={2}
          maxLength={32}
          pattern="[A-Za-z0-9_.\-]+"
          autoCapitalize="none"
          placeholder={t("users.username")}
          value={username}
          onChange={(event) => setUsername(event.target.value)}
        />
        <button type="submit" className="button button-primary" disabled={create.isPending}>
          {t("users.create")}
        </button>
      </form>
      <ErrorText error={create.error} />
      {issued && <InvitationBox issued={issued} />}
      {users.isPending && <Loading />}
      <ErrorText error={users.error} />
      <div className="list-cards">
        {users.data?.items.map((user) => (
          <UserCard key={user.id} user={user} onIssued={setIssued} />
        ))}
      </div>
    </main>
  );
}
