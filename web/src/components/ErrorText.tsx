import { useTranslation } from "react-i18next";
import { errorKey } from "../i18n";

export function ErrorText({ error }: { error: unknown }) {
  const { t } = useTranslation();
  if (!error) {
    return null;
  }
  return (
    <p className="error" role="alert">
      {t(errorKey(error))}
    </p>
  );
}
