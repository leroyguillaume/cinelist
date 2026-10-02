import { useTranslation } from "react-i18next";

export function Loading() {
  const { t } = useTranslation();
  return <p className="muted">{t("app.loading")}</p>;
}

export function Splash() {
  const { t } = useTranslation();
  return <div className="splash">{t("app.loading")}</div>;
}
