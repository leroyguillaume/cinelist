import { type FormEvent, useState } from "react";
import { useTranslation } from "react-i18next";
import { Navigate, useNavigate, useSearchParams } from "react-router";
import { ErrorText } from "../components/ErrorText";
import { useSession, useSignIn } from "../lib/queries";

/** Only same-origin paths are followed after signing in. */
export function safeNext(next: string | null): string {
  return next?.startsWith("/") && !next.startsWith("//") ? next : "/";
}

export function LoginPage() {
  const { t } = useTranslation();
  const session = useSession();
  const signIn = useSignIn();
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const next = safeNext(params.get("next"));

  if (session.data) {
    return <Navigate to={next} replace />;
  }

  const submit = (event: FormEvent) => {
    event.preventDefault();
    signIn.mutate({ username, password }, { onSuccess: () => navigate(next, { replace: true }) });
  };

  return (
    <main className="auth-screen">
      <form className="auth-card" onSubmit={submit}>
        <img className="auth-logo" src="/logo-title.svg" alt="CineList" />
        <h1 className="visually-hidden">{t("login.title")}</h1>
        <div className="field">
          <label htmlFor="username">{t("login.username")}</label>
          <input
            id="username"
            className="input"
            autoComplete="username"
            autoCapitalize="none"
            required
            value={username}
            onChange={(event) => setUsername(event.target.value)}
          />
        </div>
        <div className="field">
          <label htmlFor="password">{t("login.password")}</label>
          <input
            id="password"
            className="input"
            type="password"
            autoComplete="current-password"
            required
            value={password}
            onChange={(event) => setPassword(event.target.value)}
          />
        </div>
        <ErrorText error={signIn.error} />
        <button type="submit" className="button button-primary" disabled={signIn.isPending}>
          {t("login.submit")}
        </button>
      </form>
    </main>
  );
}
