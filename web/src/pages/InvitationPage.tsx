import { type FormEvent, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, useParams } from "react-router";
import { ErrorText } from "../components/ErrorText";
import { Splash } from "../components/Loading";
import { useAcceptInvitation, useInvitation } from "../lib/queries";

const MIN_PASSWORD_LENGTH = 8;

export function InvitationPage() {
  const { t } = useTranslation();
  const { token = "" } = useParams();
  const invitation = useInvitation(token);
  const accept = useAcceptInvitation(token);
  const navigate = useNavigate();
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [problem, setProblem] = useState<string | null>(null);

  if (invitation.isPending) {
    return <Splash />;
  }

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (password.length < MIN_PASSWORD_LENGTH) {
      setProblem(t("invitation.tooShort"));
      return;
    }
    if (password !== confirmation) {
      setProblem(t("invitation.mismatch"));
      return;
    }
    setProblem(null);
    accept.mutate(password, { onSuccess: () => navigate("/", { replace: true }) });
  };

  return (
    <main className="auth-screen">
      <div className="auth-card">
        <img className="auth-logo" src="/logo-title.svg" alt="CineList" />
        {invitation.isError ? (
          <p className="notice">{t("invitation.invalid")}</p>
        ) : (
          <form className="stack" onSubmit={submit}>
            <h1>{t("invitation.title", { username: invitation.data.username })}</h1>
            <p className="muted">{t("invitation.intro")}</p>
            <div className="field">
              <label htmlFor="password">{t("invitation.password")}</label>
              <input
                id="password"
                className="input"
                type="password"
                autoComplete="new-password"
                required
                minLength={MIN_PASSWORD_LENGTH}
                value={password}
                onChange={(event) => setPassword(event.target.value)}
              />
            </div>
            <div className="field">
              <label htmlFor="confirmation">{t("invitation.confirmation")}</label>
              <input
                id="confirmation"
                className="input"
                type="password"
                autoComplete="new-password"
                required
                value={confirmation}
                onChange={(event) => setConfirmation(event.target.value)}
              />
            </div>
            {problem && (
              <p className="error" role="alert">
                {problem}
              </p>
            )}
            <ErrorText error={accept.error} />
            <button type="submit" className="button button-primary" disabled={accept.isPending}>
              {t("invitation.submit")}
            </button>
          </form>
        )}
      </div>
    </main>
  );
}
