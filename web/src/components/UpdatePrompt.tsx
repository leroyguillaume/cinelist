import { useRegisterSW } from "virtual:pwa-register/react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { logger } from "../lib/logger";
import { CloseIcon } from "./Icons";

export function UpdatePrompt() {
  const { t } = useTranslation();
  const [registration, setRegistration] = useState<ServiceWorkerRegistration | undefined>();
  const {
    needRefresh: [needRefresh, setNeedRefresh],
    updateServiceWorker,
  } = useRegisterSW({
    onRegisteredSW: (_url, registered) => setRegistration(registered),
    onNeedRefresh: () => logger.info("pwa.update_available"),
    onRegisterError: (error: unknown) =>
      logger.warn("pwa.register_failed", { error: String(error) }),
  });

  // The browser looks for a new service worker on navigation only, and an
  // installed app can stay open for days: check again whenever it comes back.
  useEffect(() => {
    if (!registration) {
      return;
    }
    const check = () => {
      if (document.visibilityState !== "visible") {
        return;
      }
      registration.update().catch((error: unknown) => {
        logger.debug("pwa.update_check_failed", { error: String(error) });
      });
    };
    document.addEventListener("visibilitychange", check);
    return () => document.removeEventListener("visibilitychange", check);
  }, [registration]);

  if (!needRefresh) {
    return null;
  }
  return (
    <div className="update-prompt" role="status">
      <p className="update-prompt-text">{t("update.available")}</p>
      <button
        type="button"
        className="button button-primary button-small"
        onClick={() => void updateServiceWorker()}
      >
        {t("update.reload")}
      </button>
      <button
        type="button"
        className="icon-button"
        aria-label={t("update.later")}
        onClick={() => setNeedRefresh(false)}
      >
        <CloseIcon size={18} />
      </button>
    </div>
  );
}
