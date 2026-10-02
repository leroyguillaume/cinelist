import { useTranslation } from "react-i18next";
import { Link } from "react-router";

export function NotFoundPage() {
  const { t } = useTranslation();
  return (
    <main className="page">
      <p>{t("app.notFound")}</p>
      <Link to="/">{t("app.home")}</Link>
    </main>
  );
}
